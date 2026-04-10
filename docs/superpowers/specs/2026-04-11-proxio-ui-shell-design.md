## Proxio UI Shell Design

### Goal

Build the first GUI shell for Proxio as a native Linux desktop application using `iced`, centered on a single window that exposes the existing core proxy workflows: viewing the current profile, switching profiles, applying or disabling proxy configuration, and running URL checks.

### Non-Goals

- No tray integration in this phase
- No background daemon or always-on service
- No multi-window UI
- No automatic profile switching
- No full settings editor for every adapter target
- No complex navigation system or page router
- No direct GUI-side system mutations outside existing application services

### User Value

The CLI is now functional for profile management and diagnosis, but many desktop users want a native control surface for everyday operations. A GUI shell makes Proxio faster to use for common tasks without weakening the rule that all behavior must remain available and understandable through the underlying core, adapters, diagnose, and CLI layers.

### Technical Impact

This change adds a new crate, `proxio-ui`, using `iced`. It must remain a thin interaction layer over the existing workspace crates. The UI will introduce a small internal service layer so view logic stays separate from config, adapter, and diagnosis logic.

### Risks

- GUI code can easily absorb business logic if boundaries are not enforced
- `iced` integration can introduce significant compile time and dependency weight
- Async UI actions for apply and check can become tangled if state transitions are not kept simple
- A too-ambitious layout could violate the project principle of avoiding a large desktop shell too early

### Recommended Approach

Build a single-window shell with a narrow, task-oriented layout and a thin service boundary inside `proxio-ui`.

`proxio-ui` should own window state, event handling, and rendering. All config reads, profile changes, apply or disable actions, and diagnose requests should be routed through UI-local services that delegate to `proxio-core`, `proxio-adapters`, and `proxio-diagnose`.

Alternative approaches were considered:

- A full multi-page desktop application: rejected because it is too large for the current product stage
- Embedding all logic directly in the `iced` application update loop: rejected because it would collapse crate boundaries and make testing difficult
- Waiting to build any GUI until all future backend capabilities are complete: rejected because the current backend is already strong enough for a useful shell MVP

### Scope

This phase adds:

- `proxio-ui` crate based on `iced`
- one single-window application shell
- current profile summary
- profile list and profile switching
- apply and disable actions
- URL check input and layered diagnosis result display

This phase does not add:

- profile creation or deletion from the GUI
- advanced settings editing for proxy values
- tray icon behavior
- persistent background tasks
- rich visual theming work beyond a clear native MVP

### Architecture

#### Workspace Layout

The Cargo workspace should gain:

- `proxio-ui`

The existing crates remain the source of truth:

- `proxio-core`
  - config model, current profile, profile listing, validation
- `proxio-adapters`
  - apply and disable side effects
- `proxio-diagnose`
  - URL check and layered diagnosis model
- `proxio-ui`
  - state, rendering, user interaction, and thin application services

#### proxio-ui Internal Structure

`proxio-ui` should stay small and split by responsibility:

- `main.rs`
  - native app entry point
- `app.rs`
  - `iced` application state machine, messages, and rendering
- `services.rs`
  - thin wrapper around current config reads, profile switching, apply or disable actions, and diagnosis execution

The UI crate should not duplicate config rules or adapter logic.

### Window Layout

The first version uses one vertically stacked window with four sections:

1. **Profile Summary**
   - active profile name
   - mode summary such as `direct` or `proxied`

2. **Profiles**
   - list of stored profiles
   - selection control
   - `Use` button to switch the active profile without auto-applying

3. **Actions**
   - `Apply` button
   - `Disable` button
   - most recent action summary showing success, skipped, or failed target counts

4. **Check URL**
   - URL input field
   - `Check` button
   - DNS, TCP, TLS, and HTTP result rows

The layout should remain compact and functional rather than decorative.

### UI State Model

The UI should maintain at least this state:

- loaded config snapshot
- list of profile names
- current profile name
- currently selected profile in the UI list
- last action result summary
- check URL input string
- latest diagnosis result model
- current busy state for long-running actions such as apply, disable, and check
- latest user-visible error message

The busy state should be explicit so the UI can disable duplicate button clicks while an operation is running.

### Interaction Rules

#### Startup

On startup, the application should:

1. read Proxio config
2. load the active profile and profile list
3. render the current shell state

If config cannot be loaded, the UI should show a clear error instead of crashing.

#### Profile Switching

- selecting a profile and pressing `Use` should only update `current_profile`
- it must not auto-apply changes
- after success, the summary section should refresh to the newly active profile

#### Apply

- pressing `Apply` should call the existing apply flow
- results should be summarized back into the Actions section
- failures must remain visible and not be silently collapsed

#### Disable

- pressing `Disable` should call the existing disable flow
- the UI must make clear that this clears Proxio-managed proxy settings, not stored profiles

#### Check

- if the URL input is invalid, the UI should report the error immediately
- if valid, the UI should run the same diagnosis capability already exposed by `proxio check <url>`
- results must preserve the four-layer structure: DNS, TCP, TLS, HTTP

### Service Layer Responsibilities

The UI-local service layer should provide methods equivalent to:

- load current app state from config
- switch active profile
- apply current profile
- disable managed proxy settings
- run URL check for the current profile

The service layer may perform simple aggregation or formatting for the UI state, but it must not become a second business logic layer.

### Error Handling

The UI must surface clear user-facing errors for at least:

- missing or malformed config
- no current profile selected
- apply failures
- disable failures
- invalid check URL
- diagnosis failures

Error handling should prefer explicit text feedback inside the window rather than panics or hidden logs.

### Testing Strategy

#### proxio-ui logic tests

Focus on state transitions rather than pixel-accurate rendering:

- startup state loading
- profile selection and use flow
- action result mapping after apply or disable
- diagnosis result mapping into UI state
- error state transitions

#### Service tests

Use fake or test implementations where possible so the UI crate can verify behavior without touching real user config or the external network.

#### Manual verification

Manual smoke testing should confirm:

- app launches successfully
- current profile is visible
- switching profile updates summary
- apply and disable trigger visible result summaries
- entering a URL and pressing `Check` displays DNS, TCP, TLS, and HTTP rows

### Deliverables

The implementation should produce:

- `proxio-ui` crate using `iced`
- single-window shell MVP
- current profile summary section
- profile list and `Use` flow
- apply and disable controls
- URL check input and four-layer result display
- logic tests for UI state and service integration
