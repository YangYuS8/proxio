## Proxio Libcosmic UI Design

### Goal

Replace the current `iced`-based GUI shell with a `libcosmic`-based native Linux interface that better matches the visual language of COSMIC Settings while preserving the current product scope: profile overview, profile switching, apply and disable actions, URL checking, and layered diagnosis display.

### Non-Goals

- No expansion of proxy-management feature scope in this phase
- No tray integration or background daemon
- No automatic profile switching
- No full profile creation or deletion flows in the GUI
- No redesign of core, adapters, or diagnose business logic
- No custom theme engine beyond using `libcosmic` conventions effectively

### User Value

The current GUI shell is functionally useful but visually minimal and does not fit the intended product feel. Users of Linux desktop environments, especially those familiar with COSMIC and modern system settings apps, expect stronger information hierarchy, cleaner spacing, and a more native settings-panel experience. This phase improves usability and product fit without changing what the application fundamentally does.

### Technical Impact

This change keeps the `proxio-ui` crate but migrates its UI implementation from `iced` patterns to `libcosmic`. It also adds a root `justfile` for common developer workflows. The migration may require refactoring UI-local modules and state organization so the code fits `libcosmic` idioms cleanly rather than mechanically wrapping the current `iced` structure.

### Risks

- A direct one-to-one port from `iced` could preserve the wrong internal abstractions and make the code awkward
- `libcosmic` introduces a different application model that may require larger UI-layer refactoring than a pure visual refresh
- Dependency and build behavior may change significantly compared with the current GUI crate
- If too much product logic moves into UI code during migration, crate boundaries will degrade

### Recommended Approach

Treat this as a UI-layer migration and UI-structure refactor, not as a simple component swap.

The existing behavior surface should stay constant, but the `proxio-ui` internal architecture should be adjusted where needed so `libcosmic` can present the application as a settings-style control panel with better sectioning and interaction hierarchy. The thin service layer remains the integration boundary to `proxio-core`, `proxio-adapters`, and `proxio-diagnose`.

Alternative approaches were considered:

- Continue iterating on `iced` visuals only: rejected because the user explicitly wants a `libcosmic` direction and the current feel is structurally wrong
- Maintain both `iced` and `libcosmic` UIs in parallel: rejected because it doubles UI maintenance too early
- Delay UI migration and only add a `justfile`: rejected because it does not address the primary product feedback

### Scope

This phase adds or changes:

- `proxio-ui` migrated to `libcosmic`
- UI-layer refactor to better fit `libcosmic`
- COSMIC-style single-window settings layout
- root `justfile` for common development commands

This phase does not add:

- new proxy adapters
- new profile-management capabilities
- new diagnose layers or output formats
- tray, daemon, or background process behavior

### Architecture

#### Workspace Structure

The workspace continues to use:

- `proxio-core`
- `proxio-adapters`
- `proxio-diagnose`
- `proxio-ui`
- `proxio-cli`

`proxio-ui` remains a dedicated crate and must not absorb business logic from other layers.

#### proxio-ui Internal Structure

The migration should allow UI-layer refactoring. The crate may be reorganized as needed, but the responsibilities must remain clear:

- application shell and window integration
- UI state and event handling
- UI rendering and layout composition
- thin service calls into existing backend crates

The service boundary should remain explicit so apply, disable, profile switching, and URL check operations still route through the same underlying domain and adapter logic.

### UI Layout Target

The target feel is:

- `COSMIC Settings`
- modern Linux system settings
- light dashboard characteristics, but still a settings-style tool rather than a monitoring console

#### Window Structure

The single window should be reorganized into a stronger settings-style hierarchy:

1. **Header area**
   - application title
   - active profile name
   - current mode summary such as `direct` or `proxied`

2. **Main content area with two-column emphasis**
   - left side:
     - profile list
     - active and selected state
     - `Use` action
   - right side:
     - action cards
     - `Apply`
     - `Disable`
     - last action summary

3. **Diagnosis section**
   - URL entry row
   - `Check` action
   - layered DNS, TCP, TLS, and HTTP result cards or rows

The layout should read like a settings panel with grouped cards, not like stacked debug controls.

### Visual Style Requirements

The UI should move away from plain vertical button-and-text stacking and toward:

- clearer visual grouping with section containers or cards
- stronger spacing and padding
- more readable hierarchy between titles, labels, and status lines
- concise action surfaces rather than flat control lists
- diagnosis results presented as state panels, not raw log-like lines

The design should still remain lightweight and avoid decorative complexity that fights native desktop expectations.

### Interaction Rules

The behavioral rules remain unchanged from the current shell:

- `Use`
  - only switches `current_profile`
  - does not auto-apply
- `Apply`
  - applies current profile and reports success, skipped, and failed counts
- `Disable`
  - clears Proxio-managed proxy settings without deleting profiles
- `Check`
  - validates URL input
  - displays DNS, TCP, TLS, and HTTP results distinctly

The migration must preserve these semantics.

### Refactor Boundary

This phase explicitly allows UI refactoring where it improves `libcosmic` fit:

- reorganizing UI-local modules
- reshaping UI state types
- renaming UI-local structs and message enums
- rebuilding layout composition from scratch

This phase must not:

- rewrite `proxio-core`, `proxio-adapters`, or `proxio-diagnose` logic unnecessarily
- move backend logic into the UI
- change CLI-first product behavior contracts

### justfile

A root `justfile` should be added to improve local development ergonomics.

The first version should include at least:

- `just fmt`
- `just test`
- `just check`
- `just cli`
- `just ui`

The file should stay small, obvious, and focused on high-frequency commands.

### Testing Strategy

#### proxio-ui

Continue to focus on state and service interaction logic rather than pixel-perfect visual testing:

- profile selection transitions
- loaded state transitions
- action result mapping
- diagnosis result mapping
- error state transitions

#### Manual verification

Manual smoke verification should confirm:

- GUI launches through the new `libcosmic` entry point
- active profile appears in the header area
- selecting and switching a profile updates the summary area
- apply and disable results show clearly
- checking a URL displays DNS, TCP, TLS, and HTTP sections

#### justfile

Manually verify each defined recipe executes the intended command.

### Deliverables

The implementation should produce:

- `proxio-ui` migrated to `libcosmic`
- refactored UI shell with COSMIC-style grouping and hierarchy
- preserved integration with current profile, apply, disable, and check flows
- root `justfile` with common development commands
- passing tests, checks, and GUI startup smoke verification
