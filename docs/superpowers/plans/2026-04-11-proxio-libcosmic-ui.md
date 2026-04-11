# Proxio Libcosmic UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate `proxio-ui` from `iced` to `libcosmic`, rebuild the GUI as a COSMIC-style settings panel, and add a root `justfile` for common development commands.

**Architecture:** Keep `proxio-ui` as a thin UI layer over `proxio-core`, `proxio-adapters`, and `proxio-diagnose`, but refactor the UI internals to fit `libcosmic` instead of mechanically porting `iced` structures. Add a small `justfile` at the repository root for common workflows without changing backend crate boundaries.

**Tech Stack:** Rust 2024, `libcosmic`, Cargo workspace, `just`, existing workspace crates (`proxio-core`, `proxio-adapters`, `proxio-diagnose`, `proxio-cli`)

---

### Planned File Structure

**Workspace**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `justfile`

**proxio-ui**
- Modify: `crates/proxio-ui/Cargo.toml`
- Modify: `crates/proxio-ui/src/main.rs`
- Modify: `crates/proxio-ui/src/app.rs`
- Modify: `crates/proxio-ui/src/lib.rs`
- Modify: `crates/proxio-ui/src/services.rs`
- Modify: `crates/proxio-ui/tests/app_state.rs`

### Task 1: Add `justfile` and Prepare `proxio-ui` for Migration

**Files:**
- Create: `justfile`
- Modify: `crates/proxio-ui/Cargo.toml`
- Modify: `crates/proxio-ui/src/lib.rs`

- [ ] **Step 1: Write the failing workflow expectation**

The repository root should provide these commands via `just`:

```text
just fmt
just test
just check
just cli
just ui
```

And `proxio-ui` should stop depending on `iced` directly.

- [ ] **Step 2: Verify the current repository lacks `justfile`**

Run: `just --list`
Expected: FAIL because `justfile` does not exist yet

- [ ] **Step 3: Create the root `justfile`**

Create `justfile` with:

```make
fmt:
	cargo fmt --all

test:
	cargo test

check:
	cargo check

cli:
	cargo run -p proxio -- --help

ui:
	cargo run -p proxio-ui
```

- [ ] **Step 4: Replace `iced` dependency with `libcosmic` in `proxio-ui`**

Update `crates/proxio-ui/Cargo.toml` to remove `iced` and add the minimal `libcosmic` dependency set required for a basic app window.

Use the actual crate names needed by the selected `libcosmic` API, but keep the dependency list as small as possible.

- [ ] **Step 5: Keep crate exports compiling during migration**

Ensure `crates/proxio-ui/src/lib.rs` continues to expose `app` and `services` while the UI entry point is rewritten.

- [ ] **Step 6: Verify `justfile` and crate manifest wiring**

Run:

```bash
just --list
cargo check -p proxio-ui
```

Expected: `just --list` shows the new recipes, and `cargo check -p proxio-ui` may still fail until the app migration is completed, but dependency resolution should be correct.

### Task 2: Refactor UI State for COSMIC-Style Layout

**Files:**
- Modify: `crates/proxio-ui/src/app.rs`
- Modify: `crates/proxio-ui/tests/app_state.rs`

- [ ] **Step 1: Write failing state tests for the refactored UI model**

Extend `crates/proxio-ui/tests/app_state.rs` with assertions that the state can represent:

- header summary text
- profile selection and active profile independently
- action summary cards
- diagnosis section rows
- user-visible error banner

Add a focused test like:

```rust
#[test]
fn updates_summary_and_error_banner_state() {
    let mut state = AppState::default();
    state.set_header_summary(Some("proxy".into()), "proxied".into());
    state.set_error("load failed".into());

    assert_eq!(state.current_profile.as_deref(), Some("proxy"));
    assert_eq!(state.mode_summary, "proxied");
    assert_eq!(state.error_message.as_deref(), Some("load failed"));
}
```

- [ ] **Step 2: Run the UI state tests to verify they fail**

Run: `cargo test -p proxio-ui --test app_state`
Expected: FAIL until the new helper methods and state layout exist

- [ ] **Step 3: Refactor `AppState` for settings-panel structure**

In `crates/proxio-ui/src/app.rs`, reshape the state so it can cleanly support:

- header summary
- selected profile list state
- action card summary
- diagnosis panel state
- error banner state

You may keep the current fields if they still fit, but add focused helper methods such as:

```rust
impl AppState {
    pub fn set_header_summary(&mut self, profile: Option<String>, mode_summary: String) { ... }
    pub fn clear_error(&mut self) { ... }
}
```

- [ ] **Step 4: Re-run the UI state tests**

Run: `cargo test -p proxio-ui --test app_state`
Expected: PASS

### Task 3: Replace the `iced` Shell with a `libcosmic` Application Shell

**Files:**
- Modify: `crates/proxio-ui/src/app.rs`
- Modify: `crates/proxio-ui/src/main.rs`

- [ ] **Step 1: Write a failing compile expectation for the new UI entry point**

The crate should compile with a `libcosmic`-backed `run()` entry point and no `iced::application(...)` usage.

- [ ] **Step 2: Run cargo check to verify the old shell is still incompatible with the new dependency**

Run: `cargo check -p proxio-ui`
Expected: FAIL until the `iced`-based app entry is replaced

- [ ] **Step 3: Implement a minimal `libcosmic` app shell**

In `crates/proxio-ui/src/app.rs`, replace the existing `iced` builder with the corresponding `libcosmic` application pattern.

Requirements:

- one main window
- startup load task
- message enum for load, profile select, use, apply, disable, and check
- no multi-page routing

The shell should still hold a state struct and a service instance.

- [ ] **Step 4: Keep startup behavior intact**

On startup, the app should still:

- initialize services
- load profile names and active profile
- update the header summary
- display a user-visible error if config load fails

- [ ] **Step 5: Verify crate compilation**

Run: `cargo check -p proxio-ui`
Expected: PASS

### Task 4: Rebuild the Layout in a COSMIC Settings Style

**Files:**
- Modify: `crates/proxio-ui/src/app.rs`

- [ ] **Step 1: Write a failing layout-oriented state test**

Add a small state-level test in `crates/proxio-ui/tests/app_state.rs` ensuring the diagnosis and action areas can both be populated at once without clobbering each other.

- [ ] **Step 2: Run the UI test target to verify it fails**

Run: `cargo test -p proxio-ui --test app_state`
Expected: FAIL until the supporting state logic is updated

- [ ] **Step 3: Rebuild the view composition around grouped sections**

In `app.rs`, rebuild the rendered layout so it reads as:

- header area
- left profile column
- right action card column
- bottom diagnosis section

The resulting UI should:

- use clearer section titles
- group content inside settings-style containers
- avoid flat button stacks and raw log-style text dumps

- [ ] **Step 4: Present diagnosis results as state panels**

Display DNS, TCP, TLS, and HTTP as distinct grouped lines or cards with:

- layer label
- status
- short summary
- optional detail text

- [ ] **Step 5: Re-run UI tests and crate check**

Run:

```bash
cargo test -p proxio-ui --test app_state
cargo check -p proxio-ui
```

Expected: PASS

### Task 5: Preserve Existing Behavior Through the New Shell

**Files:**
- Modify: `crates/proxio-ui/src/app.rs`
- Modify: `crates/proxio-ui/src/services.rs`

- [ ] **Step 1: Write a failing behavior test for message-driven transitions**

Extend `app_state` tests so they verify the UI state can still represent:

- profile switch without auto-apply
- apply result summary
- disable result summary
- check result update

- [ ] **Step 2: Run the UI state tests to verify a red phase**

Run: `cargo test -p proxio-ui --test app_state`
Expected: FAIL until transition helpers and message handling are adjusted to the refactored shell

- [ ] **Step 3: Rewire the message handling**

Ensure the `libcosmic` update loop preserves these semantics:

- selecting a profile only changes selected state
- pressing `Use` changes `current_profile` but does not apply
- pressing `Apply` updates the action summary area
- pressing `Disable` updates the action summary area
- pressing `Check` validates URL input and updates the layered result section

- [ ] **Step 4: Keep services thin**

If `services.rs` needs adjustment for the `libcosmic` shell, keep it limited to:

- loading state
- switching current profile
- apply and disable aggregation
- running `check`

Do not move business logic into the app layer.

- [ ] **Step 5: Re-run UI tests and crate check**

Run:

```bash
cargo test -p proxio-ui --test app_state
cargo check -p proxio-ui
```

Expected: PASS

### Task 6: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-04-11-proxio-libcosmic-ui.md`

- [ ] **Step 1: Run `just` commands and full workspace verification**

Run:

```bash
just fmt
just test
just check
just --list
```

Expected: PASS

- [ ] **Step 2: Run a GUI startup smoke test**

Run:

```bash
cargo run -p proxio-ui
```

Expected: application starts without immediate crash

- [ ] **Step 3: Leave changes uncommitted unless the user later asks for a commit**

Do not create a commit during implementation unless the user explicitly asks for one.
