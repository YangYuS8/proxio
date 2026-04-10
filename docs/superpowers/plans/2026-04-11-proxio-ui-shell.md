# Proxio UI Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first native `iced` GUI shell for Proxio with a single window for profile state, profile switching, apply/disable actions, and URL check display.

**Architecture:** Add a new `proxio-ui` crate to the workspace. Keep `proxio-ui` split into a small `app` state machine and a thin `services` layer that delegates to `proxio-core`, `proxio-adapters`, and `proxio-diagnose`; do not move domain logic into the UI.

**Tech Stack:** Rust 2024, Cargo workspace, `iced`, `tokio`, existing workspace crates (`proxio-core`, `proxio-adapters`, `proxio-diagnose`)

---

### Planned File Structure

**Workspace**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**proxio-ui**
- Create: `crates/proxio-ui/Cargo.toml`
- Create: `crates/proxio-ui/src/main.rs`
- Create: `crates/proxio-ui/src/app.rs`
- Create: `crates/proxio-ui/src/services.rs`
- Create: `crates/proxio-ui/tests/app_state.rs`

### Task 1: Bootstrap `proxio-ui` Crate and Service Data Model

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/proxio-ui/Cargo.toml`
- Create: `crates/proxio-ui/src/main.rs`
- Create: `crates/proxio-ui/src/app.rs`
- Create: `crates/proxio-ui/src/services.rs`
- Create: `crates/proxio-ui/tests/app_state.rs`

- [ ] **Step 1: Write the failing UI state test**

Create `crates/proxio-ui/tests/app_state.rs` with:

```rust
use proxio_diagnose::{CheckReport, EffectiveProxy, LayerReport, LayerStatus, TransportMode};
use proxio_ui::app::{ActionSummary, AppState};

#[test]
fn loads_profile_names_and_current_selection_into_state() {
    let state = AppState::from_loaded(
        vec!["direct".into(), "proxy".into()],
        Some("proxy".into()),
        "proxied".into(),
    );

    assert_eq!(state.profile_names, vec!["direct", "proxy"]);
    assert_eq!(state.current_profile.as_deref(), Some("proxy"));
    assert_eq!(state.selected_profile.as_deref(), Some("proxy"));
}

#[test]
fn records_action_summary_and_check_report() {
    let mut state = AppState::default();
    state.action_summary = Some(ActionSummary {
        success_count: 2,
        skipped_count: 1,
        failed_count: 0,
        message: "applied profile".into(),
    });
    state.last_check = Some(CheckReport {
        target_url: "https://example.com".into(),
        profile_name: "proxy".into(),
        transport: EffectiveProxy {
            mode: TransportMode::Proxied,
            value: Some("http://127.0.0.1:7890".into()),
        },
        dns: LayerReport { status: LayerStatus::Success, summary: "resolved".into(), detail: String::new() },
        tcp: LayerReport { status: LayerStatus::Success, summary: "connected".into(), detail: String::new() },
        tls: LayerReport { status: LayerStatus::Success, summary: "tls ok".into(), detail: String::new() },
        http: LayerReport { status: LayerStatus::Success, summary: "http ok".into(), detail: String::new() },
        conclusion: "ok".into(),
    });

    assert_eq!(state.action_summary.as_ref().unwrap().success_count, 2);
    assert_eq!(state.last_check.as_ref().unwrap().profile_name, "proxy");
}
```

- [ ] **Step 2: Run the new test target to verify it fails**

Run: `cargo test -p proxio-ui --test app_state`
Expected: FAIL because the crate and types do not exist yet

- [ ] **Step 3: Add the workspace member and crate manifest**

Update root `Cargo.toml` members to include `crates/proxio-ui`.

Create `crates/proxio-ui/Cargo.toml`:

```toml
[package]
name = "proxio-ui"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
iced = { version = "0.13", features = ["tokio"] }
proxio-core = { path = "../proxio-core" }
proxio-adapters = { path = "../proxio-adapters" }
proxio-diagnose = { path = "../proxio-diagnose" }
tokio = { version = "1", features = ["rt", "sync"] }

[dev-dependencies]
proxio-diagnose = { path = "../proxio-diagnose" }
```

- [ ] **Step 4: Add the initial service and app state skeleton**

Create `crates/proxio-ui/src/app.rs` with:

```rust
use proxio_diagnose::CheckReport;

#[derive(Debug, Clone, Default)]
pub struct ActionSummary {
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub profile_names: Vec<String>,
    pub current_profile: Option<String>,
    pub selected_profile: Option<String>,
    pub mode_summary: String,
    pub action_summary: Option<ActionSummary>,
    pub check_input: String,
    pub last_check: Option<CheckReport>,
    pub is_busy: bool,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn from_loaded(
        profile_names: Vec<String>,
        current_profile: Option<String>,
        mode_summary: String,
    ) -> Self {
        Self {
            selected_profile: current_profile.clone(),
            profile_names,
            current_profile,
            mode_summary,
            ..Self::default()
        }
    }
}
```

Create `crates/proxio-ui/src/services.rs` with service-facing structs only:

```rust
use proxio_diagnose::CheckReport;

#[derive(Debug, Clone)]
pub struct LoadedState {
    pub profile_names: Vec<String>,
    pub current_profile: Option<String>,
    pub mode_summary: String,
}

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub message: String,
}

pub trait UiServices {
    fn load(&self) -> Result<LoadedState, String>;
    fn use_profile(&self, name: &str) -> Result<LoadedState, String>;
    fn apply(&self) -> Result<ActionResult, String>;
    fn disable(&self) -> Result<ActionResult, String>;
    fn check(&self, url: &str) -> Result<CheckReport, String>;
}
```

- [ ] **Step 5: Add minimal entry point and export the library API**

Create `crates/proxio-ui/src/main.rs`:

```rust
fn main() -> iced::Result {
    proxio_ui::app::run()
}
```

Also create `crates/proxio-ui/src/lib.rs` with:

```rust
pub mod app;
pub mod services;
```

- [ ] **Step 6: Run the new test target**

Run: `cargo test -p proxio-ui --test app_state`
Expected: PASS

### Task 2: Implement Real UI Services over Existing Crates

**Files:**
- Modify: `crates/proxio-ui/src/services.rs`

- [ ] **Step 1: Write a failing service smoke expectation**

The service layer should compile with a concrete service type that can:

- load current config state
- switch profile
- apply current profile
- disable managed settings
- run URL check

- [ ] **Step 2: Run cargo check to verify the concrete service type does not exist yet**

Run: `cargo check -p proxio-ui`
Expected: FAIL if you reference the concrete service before implementation

- [ ] **Step 3: Implement a concrete service type**

Extend `services.rs` with a minimal concrete type:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use proxio_core::ProxioConfig;

pub struct RealServices {
    root: PathBuf,
}

impl RealServices {
    pub fn new() -> Result<Self, String> {
        let root = if let Ok(path) = std::env::var("PROXIO_HOME") {
            PathBuf::from(path)
        } else {
            PathBuf::from(std::env::var("HOME").map_err(|err| err.to_string())?)
        };
        Ok(Self { root })
    }
}
```

- [ ] **Step 4: Add config helpers and action aggregation**

In the same file, implement helper functions equivalent to the CLI logic:

```rust
fn config_path(root: &Path) -> PathBuf {
    proxio_adapters::paths::proxio_config_dir(root).join("config.toml")
}

fn read_config(path: &Path) -> Result<ProxioConfig, String> {
    let content = fs::read_to_string(path).map_err(|err| format!("failed to read {}: {}", path.display(), err))?;
    toml::from_str(&content).map_err(|err| format!("failed to parse {}: {}", path.display(), err))
}

fn write_config(path: &Path, config: &ProxioConfig) -> Result<(), String> {
    let content = toml::to_string_pretty(config).map_err(|err| err.to_string())?;
    proxio_adapters::file_ops::atomic_write(path, &content)
}
```

Add a helper that converts adapter results into `ActionResult` counts.

- [ ] **Step 5: Implement `UiServices` for `RealServices`**

Use the existing crates:

- `load()`
  - read config
  - derive profile names, current profile, and mode summary
- `use_profile(name)`
  - update `current_profile`
  - persist config
  - return refreshed `LoadedState`
- `apply()`
  - build current profile plan and call `proxio_adapters::apply_plan`
- `disable()`
  - build disable plan and call `proxio_adapters::apply_plan`
- `check(url)`
  - resolve current profile and call `proxio_diagnose::build_check_report(..., &proxio_diagnose::RealRunner)`

- [ ] **Step 6: Run UI crate verification**

Run:

```bash
cargo test -p proxio-ui --test app_state
cargo check -p proxio-ui
```

Expected: PASS

### Task 3: Implement the `iced` Single-Window Shell

**Files:**
- Modify: `crates/proxio-ui/src/app.rs`
- Modify: `crates/proxio-ui/src/main.rs`

- [ ] **Step 1: Write a failing compile expectation for the UI app entry**

The UI crate should compile with a callable `run()` function that launches the `iced` shell.

- [ ] **Step 2: Run cargo check to verify the app entry is still incomplete**

Run: `cargo check -p proxio-ui`
Expected: FAIL if `run()` or the shell type is not implemented yet

- [ ] **Step 3: Add the `iced` message enum and shell struct**

In `app.rs`, define:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<crate::services::LoadedState, String>),
    ProfileSelected(String),
    UseSelectedProfile,
    UsedProfile(Result<crate::services::LoadedState, String>),
    ApplyPressed,
    Applied(Result<crate::services::ActionResult, String>),
    DisablePressed,
    Disabled(Result<crate::services::ActionResult, String>),
    CheckInputChanged(String),
    CheckPressed,
    Checked(Result<proxio_diagnose::CheckReport, String>),
}
```

Create a shell struct holding `AppState` and `RealServices`.

- [ ] **Step 4: Implement a minimal `iced` application builder**

Use the current `iced 0.13` builder style, keeping it simple:

```rust
pub fn run() -> iced::Result {
    iced::application("Proxio", update, view).run_with(|| {
        let services = crate::services::RealServices::new().expect("services init");
        (
            Shell {
                state: AppState::default(),
                services,
            },
            iced::Task::perform(async { crate::services::RealServices::new().and_then(|svc| svc.load()) }, Message::Loaded),
        )
    })
}
```

If needed, adjust so the load task uses the existing service instance data instead of duplicating setup.

- [ ] **Step 5: Implement the layout sections in `view()`**

Render one vertical column with four grouped sections:

- current profile summary text
- profile list buttons or selectable rows
- apply/disable buttons and action summary text
- URL input, `Check` button, and diagnosis result rows

Keep the view simple and text-heavy. Do not add multi-page routing.

- [ ] **Step 6: Run UI crate build verification**

Run: `cargo check -p proxio-ui`
Expected: PASS

### Task 4: Wire State Transitions for Use, Apply, Disable, and Check

**Files:**
- Modify: `crates/proxio-ui/src/app.rs`

- [ ] **Step 1: Extend the existing state test for event-driven transitions**

Add a focused test in `crates/proxio-ui/tests/app_state.rs` that validates `AppState` mutation helpers or event handlers produce:

- selected profile updates
- action summary updates
- check result updates
- error state updates

Keep this as pure state logic; do not require a live `iced` runtime.

- [ ] **Step 2: Run the UI test target to verify the new transition assertions fail**

Run: `cargo test -p proxio-ui --test app_state`
Expected: FAIL until transition helpers exist

- [ ] **Step 3: Add small state helper methods**

In `app.rs`, add focused methods such as:

```rust
impl AppState {
    pub fn set_selected_profile(&mut self, name: String) { ... }
    pub fn set_loaded(&mut self, loaded: crate::services::LoadedState) { ... }
    pub fn set_action_result(&mut self, result: crate::services::ActionResult) { ... }
    pub fn set_check_result(&mut self, report: proxio_diagnose::CheckReport) { ... }
    pub fn set_error(&mut self, message: String) { ... }
}
```

- [ ] **Step 4: Wire the `update()` function**

Handle messages so that:

- `Loaded` fills initial state
- `ProfileSelected` changes UI selection only
- `UseSelectedProfile` calls services without auto-apply
- `ApplyPressed` and `DisablePressed` call services and store action summaries
- `CheckPressed` validates input and calls services
- all failures become user-visible `error_message`

- [ ] **Step 5: Re-run UI tests and crate check**

Run:

```bash
cargo test -p proxio-ui --test app_state
cargo check -p proxio-ui
```

Expected: PASS

### Task 5: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-04-11-proxio-ui-shell.md`

- [ ] **Step 1: Run formatting and full workspace verification**

Run:

```bash
cargo fmt --all
cargo test
cargo check
```

Expected: PASS

- [ ] **Step 2: Manual smoke check the GUI entry point**

Run:

```bash
cargo run -p proxio-ui
```

Expected: the window launches without immediate crash

- [ ] **Step 3: Leave changes uncommitted unless the user later requests a commit**

Do not create a commit during implementation unless the user explicitly asks for one.
