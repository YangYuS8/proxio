## Proxio Profile Management Design

### Goal

Extend the current Proxio CLI MVP so users can define multiple proxy profiles, switch the active profile without immediately applying it, inspect the active profile, and disable all Proxio-managed proxy targets in a controlled way.

### Non-Goals

- No automatic profile switching based on network conditions
- No GUI profile management in this phase
- No NetworkManager integration in this phase
- No diagnostics in this phase
- No system-level configuration changes
- No forced deletion flow for the active profile

### User Value

The current MVP can store and apply only one proxy configuration. Real users need at least a few named states such as `direct`, `proxy`, and `lab`, and they need to switch between them safely. This phase makes Proxio practical for daily use by separating stored profiles from applied state and by adding a dedicated disable operation.

### Technical Impact

This change expands `proxio-core` from a single-profile config model to a multi-profile config model while preserving the existing `ApplyPlan` architecture. `proxio-cli` will gain profile management commands and a disable command. `proxio-adapters` will gain clear or unset behavior so the CLI can remove Proxio-managed proxy settings as explicitly as it applies them.

### Risks

- Config format migration from the current single-profile file must be handled clearly
- Disable behavior for Git, npm, and pnpm must avoid ambiguous or partial cleanup semantics
- Removing profiles must remain conservative to avoid unexpected state changes
- Users may expect `profile use` to apply immediately unless the CLI is explicit that it only changes current selection

### Recommended Approach

Keep `ApplyPlan` as the central abstraction and add profile selection above it.

`proxio-core` should own the multi-profile config model, resolution of the active profile, and generation of both apply plans and disable plans. `proxio-cli` should remain responsible only for config file updates and command routing. `proxio-adapters` should continue to consume plan operations without learning about higher-level profile concepts.

Alternative approaches were considered:

- Make adapters understand profiles directly: rejected because it breaks layering and couples side effects to config management
- Implement automatic apply on `profile use`: rejected because it creates hidden side effects and conflicts with the product rule to avoid magic behavior
- Add profile support only in the CLI by rewriting the config ad hoc: rejected because it would duplicate domain logic outside `proxio-core`

### Scope

This phase adds:

- multi-profile config in `~/.config/proxio/config.toml`
- active profile tracking
- CLI profile management commands
- `disable` to clear Proxio-managed proxy settings

This phase does not add:

- automatic scene detection
- profile import or export UX
- GUI profile editing
- systemd reload orchestration or shell session mutation

### Configuration Model

The config file should evolve from a single `proxy` section to this shape:

```toml
current_profile = "proxy"

[profiles.direct]

[profiles.proxy]
http_proxy = "http://127.0.0.1:7890"
https_proxy = "http://127.0.0.1:7890"
all_proxy = "socks5://127.0.0.1:7891"
no_proxy = ["127.0.0.1", "localhost"]

[profiles.lab]
http_proxy = "http://lab-gateway.internal:8080"
https_proxy = "http://lab-gateway.internal:8080"
```

#### Core Types

`proxio-core` should introduce:

- `ProfileName`
  - a validated profile identifier represented as a string wrapper or equivalent checked value
- `ProxioConfig`
  - `current_profile: Option<String>`
  - `profiles: BTreeMap<String, ProxySettings>`
- methods for:
  - listing profiles in stable order
  - resolving the current profile
  - building an apply plan for a named profile or the current profile
  - building a disable plan

The use of `BTreeMap` is preferred so profile display and serialization stay stable without extra sorting logic.

### Data Flow

1. `profile add`, `profile remove`, and `profile use` mutate only Proxio's config file
2. `profile current` reads config and resolves the current profile name and settings
3. `preview` reads the config, resolves the current profile, validates the selected `ProxySettings`, and builds an `ApplyPlan`
4. `apply` uses the same resolved plan and applies it through existing adapters
5. `disable` builds a dedicated disable plan and applies it through the same adapters

This keeps profile management separate from side effects while preserving preview and apply consistency.

### CLI Commands

#### Profile Commands

- `proxio profile list`
  - list all stored profiles and mark the active one
- `proxio profile add <name> [--http-proxy ... --https-proxy ... --all-proxy ... --no-proxy ...]`
  - create a new named profile
  - fail if the name already exists
- `proxio profile remove <name>`
  - remove a stored profile
  - fail if the name does not exist
  - fail if the name is the currently active profile
- `proxio profile use <name>`
  - set `current_profile`
  - do not automatically apply changes
- `proxio profile current`
  - display the active profile name and its proxy settings

#### Existing Commands

- `proxio preview`
  - preview the currently active profile
- `proxio apply`
  - apply the currently active profile

#### New Command

- `proxio disable`
  - clear all Proxio-managed proxy targets without deleting stored profiles

### Disable Semantics

Disable should be explicit and target-aware.

- shell env file
  - rewrite the managed file so it contains no proxy exports
- systemd user environment
  - rewrite the managed file so it contains no proxy assignments
- Git
  - unset `http.proxy` and `https.proxy`
- npm
  - delete or unset `proxy` and `https-proxy`
- pnpm
  - delete or unset `proxy` and `https-proxy`

The implementation should model disable as a first-class plan, not as a special CLI-only branch that bypasses adapters.

### Adapter Behavior Changes

Current `PlannedOperation` entries are simple key-value pairs. This phase should generalize them so adapters can represent both set and unset operations without guessing from missing keys.

The preferred direction is to introduce an operation value type such as:

- `Set(String)`
- `Unset`

This allows file targets to omit variables while command targets can generate the correct unset commands.

### Edge Cases

- No profiles stored
  - `profile list` should produce a valid empty result
  - `preview`, `apply`, and `profile current` should return clear errors
- `current_profile` missing or pointing to an unknown profile
  - treat as config error with a clear message
- Trying to remove the active profile
  - reject with a clear error instead of implicitly switching to another profile
- Adding a duplicate profile name
  - reject with a clear error
- Empty profile values
  - continue to normalize empty strings to absent values

### Migration Strategy

Existing users may already have a single-profile config file in the old shape:

```toml
[proxy]
http_proxy = "..."
```

`proxio-core` should accept this legacy shape during deserialization and normalize it into the new model using a default profile name such as `default`, with `current_profile` set to that profile. New writes should use only the new multi-profile format.

This keeps the MVP upgrade path simple and avoids silently breaking existing stored config.

### Failure Scenarios

- malformed config file
- unknown current profile
- invalid proxy URL inside a selected profile
- command target unavailable during apply or disable
- command target returns an error while clearing settings
- file target cannot be rewritten atomically

Failure handling rules:

- profile-resolution errors should stop before building or applying any plan
- target-specific failures during apply or disable should remain target-scoped in output
- missing optional tools should still be reported as skipped when possible

### Testing Strategy

#### proxio-core tests

- deserialize and serialize the multi-profile config format
- legacy single-profile config migration
- resolve current profile success and failure cases
- duplicate-safe profile listing order
- apply plan generation for a named profile
- disable plan generation with unset operations

#### proxio-adapters tests

- file target rendering for set and unset plans
- Git unset command generation
- npm and pnpm unset command generation
- apply and preview behavior for disable plans

#### proxio-cli tests

- `profile list`, `add`, `remove`, `use`, and `current`
- current-profile missing errors for `preview` and `apply`
- `disable` success path
- `profile use` updates config without applying side effects

### Deliverables

The implementation should produce:

- multi-profile config support in `proxio-core`
- backward-compatible read support for the old single-profile config shape
- CLI profile management commands
- `disable` support using the shared adapter pipeline
- adapter support for unset operations
- tests covering profile resolution, migration, and clear or unset behavior
