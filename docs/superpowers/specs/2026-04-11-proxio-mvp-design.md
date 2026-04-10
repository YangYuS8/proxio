## Proxio First-Phase MVP Design

### Goal

Build the first usable Proxio MVP as a Rust Cargo workspace focused on user-level proxy configuration management for Linux desktop developers. The MVP should provide a clear CLI workflow for storing proxy settings, previewing concrete changes, and applying those changes to supported user-level targets.

### Non-Goals

- No GUI in this phase
- No D-Bus integration in this phase
- No network diagnostics in this phase
- No system-level or root-required changes
- No automatic modification of large existing shell startup files such as `.zshrc` or `.bashrc`
- No profile auto-switching or scene detection
- No Docker, NetworkManager, Tailscale, WireGuard, or Niri IPC integration
- No full automatic rollback system in this phase

### User Value

The MVP gives Linux and Wayland users one consistent way to manage everyday proxy settings across common development tools. Instead of manually updating shell variables, `environment.d`, Git, and npm or pnpm separately, the user can define one proxy state, preview exactly what will happen, and then apply it in a controlled way.

### Technical Impact

The repository will gain an initial Rust Cargo workspace with three crates:

- `proxio-core`: proxy domain model, config model, validation, and apply-plan generation
- `proxio-adapters`: target-specific file and command adapters for shell, systemd user environment, Git, and npm or pnpm
- `proxio-cli`: command-line interface and orchestration

This design establishes the long-term crate boundaries defined in `openspec/config.yaml` while keeping the first implementation small and testable.

### Risks

- User environments vary, especially for npm, pnpm, and shell startup behavior
- External commands such as `git config` may fail or be unavailable
- Users may expect changes to take effect immediately in already-running shells, which this phase does not guarantee
- Without full rollback support, recovery depends on clear preview and explicit file targets

### Recommended Approach

Use a unified apply-plan model.

`proxio-core` will transform validated user configuration into a standard `ApplyPlan`. `proxio-adapters` will consume that plan and apply it to each concrete target. This keeps user-visible behavior consistent across CLI commands and avoids duplicating target-generation logic in the CLI layer.

Alternative approaches were considered:

- Direct CLI-to-adapter calls: faster initially, but duplicates logic and makes preview behavior harder to keep consistent
- Full profile-centric system first: stronger long-term abstraction, but too heavy for the MVP

### Architecture

#### Cargo Workspace

The workspace will contain:

- `proxio-core`
- `proxio-adapters`
- `proxio-cli`

Rust 2024 edition will be used by default.

#### proxio-core

Responsibilities:

- Define persisted Proxio configuration
- Define the proxy settings model
- Validate user input
- Generate a target-independent `ApplyPlan`

Planned core types:

- `ProxySettings`
  - `http_proxy: Option<String>`
  - `https_proxy: Option<String>`
  - `all_proxy: Option<String>`
  - `no_proxy: Vec<String>`
- `ProxioConfig`
  - persisted settings and future extensibility hooks
- `ApplyPlan`
  - a list of target operations describing what each adapter should write or execute
- `TargetKind`
  - shell env file
  - systemd user environment
  - Git global config
  - npm user config
  - pnpm user config

Validation rules:

- Proxy URLs must be syntactically valid when present
- Empty strings should normalize to absent values where appropriate
- `no_proxy` entries should be trimmed, deduplicated, and stored in stable order
- Configuration generation must distinguish unset values from explicitly provided values

#### proxio-adapters

Responsibilities:

- Convert target operations into concrete file writes or command executions
- Support `preview` and `apply` behavior per target
- Keep all path handling centralized
- Return explicit, target-specific errors

Initial adapters:

- `shell_env`
  - writes `~/.config/proxio/env/proxy.env`
- `systemd_user_env`
  - writes `~/.config/environment.d/proxio-proxy.conf`
- `git`
  - writes user-level proxy settings via `git config --global`
- `npm`
  - writes user-level config via `npm config set`
- `pnpm`
  - writes user-level config via `pnpm config set`

File-writing adapters must prefer atomic writes. Command adapters must expose enough information for dry-run preview and for tests using a fake command runner.

#### proxio-cli

Responsibilities:

- Parse CLI arguments
- Read and write Proxio config
- Produce human-readable summaries for `show`, `preview`, and `apply`
- Delegate all domain logic to `proxio-core` and all side effects to `proxio-adapters`

### Commands

The first-phase CLI will provide these commands:

- `proxio set`
  - store the desired proxy configuration in Proxio's config file
- `proxio show`
  - display the currently stored configuration
- `proxio preview`
  - show which targets will be changed and the resulting content or action summary for each target
- `proxio apply`
  - execute the apply plan against all supported targets

The CLI should be able to report per-target results as `success`, `skipped`, or `failed`.

### Configuration And File Targets

#### Proxio Config

Persisted Proxio config file:

- `~/.config/proxio/config.toml`

This file stores the desired proxy state, not raw snapshots of every target's current state.

#### Shell Env File

Target file:

- `~/.config/proxio/env/proxy.env`

Behavior:

- Write exported environment variables such as `http_proxy`, `https_proxy`, `all_proxy`, and `no_proxy`
- Include uppercase equivalents if the implementation decides they are needed for compatibility, but this should be an explicit code decision rather than implied behavior
- Do not automatically rewrite `.bashrc` or `.zshrc`
- CLI output should tell the user this file must be sourced or integrated into shell startup manually

#### systemd User Environment

Target file:

- `~/.config/environment.d/proxio-proxy.conf`

Behavior:

- Write user-scoped environment assignments suitable for `environment.d`
- Keep only Proxio-managed content in this file

#### Git

Behavior:

- Apply settings through `git config --global`
- Configure only user-level Git settings in this phase

#### npm And pnpm

Behavior:

- Apply user-level proxy settings through the respective CLI tools
- Missing tools should result in per-target `skipped` results rather than global failure when the target cannot be applied in the current environment

### Data Flow

1. `proxio set` accepts user input and writes `~/.config/proxio/config.toml`
2. `proxio show` reads and displays the current stored config
3. `proxio preview` reads the stored config, validates it through `proxio-core`, and builds an `ApplyPlan`
4. `proxio-adapters` convert the plan into preview output for each target
5. `proxio apply` executes the same `ApplyPlan` and returns per-target results

This keeps preview and apply behavior aligned by using the same underlying plan generation.

### Failure Scenarios

Examples of expected failures or edge cases:

- Invalid proxy URL in stored config
- Empty config with nothing to apply
- Failure to create required config directories
- Failure to atomically replace a target file
- External command error from `git`, `npm`, or `pnpm`
- `pnpm` not installed on the current machine

Failure handling rules:

- Validation failures should stop execution before any apply work begins
- Target-specific apply failures should be reported explicitly with target name and reason
- One target failing should not erase already-completed changes to other targets in this phase
- Missing optional tools should produce `skipped` when that target is unavailable in the environment

### Preview And Recovery Strategy

The MVP prioritizes inspectability before automation.

- Every supported target must provide preview information
- File targets must use atomic writes where possible
- The CLI must clearly show target file locations and command targets
- Full automatic rollback is deferred, but the structure should allow future backup or snapshot support

For this phase, safety comes from explicit previews, narrow user-level scope, and deterministic file ownership.

### Testing Strategy

#### proxio-core tests

- TOML serialization and deserialization of `ProxioConfig`
- URL validation for present proxy fields
- `no_proxy` normalization and deduplication
- `ApplyPlan` generation for complete and partial settings

#### proxio-adapters tests

- Temporary-directory tests for shell env output
- Temporary-directory tests for `environment.d` output
- Fake command runner tests for Git, npm, and pnpm command generation
- Failure-path tests for write and command errors

#### proxio-cli tests

- Command parsing tests
- Success-path tests for `set`, `show`, `preview`, and `apply`
- Error-reporting tests for validation failures and target-level failures

### Scope Check

This design is intentionally limited to the smallest useful system that matches the first-phase support list in `openspec/config.yaml`:

- user-level proxy variables
- shell env file
- systemd user environment
- Git proxy
- npm or pnpm proxy

The design does not attempt to solve GUI flows, profile automation, or diagnostics yet.

### Deliverables

The implementation should produce:

- a compilable Cargo workspace
- `proxio-core`
- `proxio-adapters`
- `proxio-cli`
- `set`, `show`, `preview`, and `apply` commands
- support for shell env, systemd user env, Git, npm, and pnpm targets
- unit tests and focused integration-style tests for configuration generation and adapter behavior
