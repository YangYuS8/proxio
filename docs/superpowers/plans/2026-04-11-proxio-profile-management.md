# Proxio Profile Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add named proxy profiles, active profile selection, legacy config migration, and a shared `disable` flow to the existing Proxio CLI MVP.

**Architecture:** Extend `proxio-core` to own the multi-profile config model, current-profile resolution, legacy config migration, and both apply and disable plan generation. Keep `proxio-adapters` focused on rendering and side effects by generalizing planned values from plain key-value pairs to explicit set or unset operations. Extend `proxio-cli` with profile subcommands and `disable` while preserving explicit apply semantics.

**Tech Stack:** Rust 2024, Cargo workspace, `clap`, `serde`, `toml`, `url`, `tempfile`

---

### Planned File Structure

**proxio-core**
- Modify: `crates/proxio-core/src/config.rs`
- Modify: `crates/proxio-core/src/plan.rs`
- Modify: `crates/proxio-core/src/lib.rs`
- Create: `crates/proxio-core/tests/profiles.rs`

**proxio-adapters**
- Modify: `crates/proxio-adapters/src/shell_env.rs`
- Modify: `crates/proxio-adapters/src/systemd_user_env.rs`
- Modify: `crates/proxio-adapters/src/git.rs`
- Modify: `crates/proxio-adapters/src/npm.rs`
- Modify: `crates/proxio-adapters/src/pnpm.rs`
- Modify: `crates/proxio-adapters/src/apply.rs`
- Modify: `crates/proxio-adapters/tests/file_targets.rs`
- Modify: `crates/proxio-adapters/tests/command_targets.rs`

**proxio-cli**
- Modify: `crates/proxio-cli/src/main.rs`

### Task 1: Add Multi-Profile Core Model

**Files:**
- Modify: `crates/proxio-core/src/config.rs`
- Modify: `crates/proxio-core/src/lib.rs`
- Create: `crates/proxio-core/tests/profiles.rs`

- [ ] **Step 1: Write the failing profile core tests**

Create `crates/proxio-core/tests/profiles.rs` with these tests:

```rust
use proxio_core::{ProxioConfig, ProxySettings};

#[test]
fn resolves_current_profile_from_multi_profile_config() {
    let config = ProxioConfig::new_with_profiles(
        Some("proxy".into()),
        [
            (
                "direct".into(),
                ProxySettings {
                    http_proxy: None,
                    https_proxy: None,
                    all_proxy: None,
                    no_proxy: vec![],
                },
            ),
            (
                "proxy".into(),
                ProxySettings {
                    http_proxy: Some("http://127.0.0.1:7890".into()),
                    https_proxy: Some("http://127.0.0.1:7890".into()),
                    all_proxy: None,
                    no_proxy: vec!["localhost".into()],
                },
            ),
        ],
    );

    let (name, profile) = config.current_profile().expect("current profile should resolve");
    assert_eq!(name, "proxy");
    assert_eq!(profile.http_proxy.as_deref(), Some("http://127.0.0.1:7890"));
}

#[test]
fn migrates_legacy_single_profile_shape() {
    let content = r#"
[proxy]
http_proxy = "http://127.0.0.1:7890"
https_proxy = "http://127.0.0.1:7890"
"#;

    let config: ProxioConfig = toml::from_str(content).expect("legacy config should deserialize");
    let (name, profile) = config.current_profile().expect("legacy current profile should resolve");
    assert_eq!(name, "default");
    assert_eq!(profile.https_proxy.as_deref(), Some("http://127.0.0.1:7890"));
}

#[test]
fn builds_disable_plan_with_unset_values() {
    let plan = ProxioConfig::build_disable_plan();
    assert_eq!(plan.operations.len(), 5);
    assert!(plan.operations.iter().all(|op| op.entries.iter().all(|entry| entry.value.is_unset())));
}
```

- [ ] **Step 2: Run core tests to verify they fail**

Run: `cargo test -p proxio-core profiles`
Expected: FAIL with missing multi-profile APIs and unset support

- [ ] **Step 3: Implement the new config model and legacy migration**

Update `crates/proxio-core/src/config.rs` to add:

```rust
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxioConfig {
    #[serde(default)]
    pub current_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProxySettings>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ConfigRepr {
    Current(ProxioConfig),
    Legacy { proxy: ProxySettings },
}

impl<'de> Deserialize<'de> for ProxioConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match ConfigRepr::deserialize(deserializer)? {
            ConfigRepr::Current(config) => Ok(config),
            ConfigRepr::Legacy { proxy } => {
                let mut profiles = BTreeMap::new();
                profiles.insert("default".into(), proxy);
                Ok(Self {
                    current_profile: Some("default".into()),
                    profiles,
                })
            }
        }
    }
}
```

- [ ] **Step 4: Add current-profile and constructor helpers**

Extend `crates/proxio-core/src/config.rs` with methods matching the tests:

```rust
impl ProxioConfig {
    pub fn new_with_profiles(
        current_profile: Option<String>,
        profiles: impl IntoIterator<Item = (String, ProxySettings)>,
    ) -> Self {
        Self {
            current_profile,
            profiles: profiles.into_iter().collect(),
        }
    }

    pub fn current_profile(&self) -> Result<(&str, &ProxySettings), ValidationError> {
        let name = self.current_profile.as_deref().ok_or_else(|| ValidationError {
            field: "current_profile",
            message: "no current profile selected".into(),
        })?;
        let settings = self.profiles.get(name).ok_or_else(|| ValidationError {
            field: "current_profile",
            message: format!("unknown profile: {name}"),
        })?;
        Ok((name, settings))
    }
}
```

- [ ] **Step 5: Export the new API and rerun the focused tests**

Expose `ProxioConfig` as before via `lib.rs`, then run:

Run: `cargo test -p proxio-core profiles`
Expected: still FAIL until plan types support unset operations

### Task 2: Generalize ApplyPlan for Set and Unset Operations

**Files:**
- Modify: `crates/proxio-core/src/plan.rs`
- Modify: `crates/proxio-core/src/config.rs`
- Modify: `crates/proxio-core/src/lib.rs`
- Modify: `crates/proxio-core/tests/config_and_plan.rs`
- Modify: `crates/proxio-core/tests/profiles.rs`

- [ ] **Step 1: Write the failing plan-shape tests**

Update `crates/proxio-core/tests/config_and_plan.rs` so it asserts explicit set values:

```rust
assert!(plan.operations.iter().all(|op| op.entries.iter().all(|entry| entry.value.is_set())));
```

Add this assertion to `builds_disable_plan_with_unset_values` in `profiles.rs`:

```rust
assert!(plan.operations.iter().all(|op| op.entries.iter().map(|entry| entry.key.as_str()).eq(["http_proxy", "https_proxy", "all_proxy", "no_proxy"])));
```

- [ ] **Step 2: Run core tests to verify they fail on the old plan shape**

Run: `cargo test -p proxio-core`
Expected: FAIL because `PlannedEntryValue` and explicit entries do not exist yet

- [ ] **Step 3: Replace key-value tuples with explicit planned entries**

Update `crates/proxio-core/src/plan.rs` to this shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedEntryValue {
    Set(String),
    Unset,
}

impl PlannedEntryValue {
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }

    pub fn is_unset(&self) -> bool {
        matches!(self, Self::Unset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedEntry {
    pub key: String,
    pub value: PlannedEntryValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedOperation {
    pub target: TargetKind,
    pub entries: Vec<PlannedEntry>,
}
```

- [ ] **Step 4: Update apply-plan generation and add disable-plan generation**

In `crates/proxio-core/src/plan.rs`, implement:

```rust
impl ApplyPlan {
    pub fn from_settings(settings: &ProxySettings) -> Self { /* build Set(...) entries */ }

    pub fn disable() -> Self {
        let entries = ["http_proxy", "https_proxy", "all_proxy", "no_proxy"]
            .into_iter()
            .map(|key| PlannedEntry {
                key: key.into(),
                value: PlannedEntryValue::Unset,
            })
            .collect::<Vec<_>>();
        /* reuse the same target list as from_settings */
    }
}
```

- [ ] **Step 5: Update config methods to use the new plan API**

Extend `crates/proxio-core/src/config.rs` with:

```rust
pub fn build_plan_for_current_profile(&self) -> Result<ApplyPlan, ValidationError> {
    let (_, profile) = self.current_profile()?;
    crate::validate::validate_proxy_settings(profile)?;
    Ok(ApplyPlan::from_settings(profile))
}

pub fn build_disable_plan() -> ApplyPlan {
    ApplyPlan::disable()
}
```

- [ ] **Step 6: Export new plan types and rerun core tests**

Export `PlannedEntry` and `PlannedEntryValue` from `lib.rs`, then run:

Run: `cargo test -p proxio-core`
Expected: PASS

### Task 3: Add Unset Rendering and Command Clearing in Adapters

**Files:**
- Modify: `crates/proxio-adapters/src/shell_env.rs`
- Modify: `crates/proxio-adapters/src/systemd_user_env.rs`
- Modify: `crates/proxio-adapters/src/git.rs`
- Modify: `crates/proxio-adapters/src/npm.rs`
- Modify: `crates/proxio-adapters/src/pnpm.rs`
- Modify: `crates/proxio-adapters/src/apply.rs`
- Modify: `crates/proxio-adapters/tests/file_targets.rs`
- Modify: `crates/proxio-adapters/tests/command_targets.rs`

- [ ] **Step 1: Write the failing adapter tests for disable plans**

Add this test to `crates/proxio-adapters/tests/file_targets.rs`:

```rust
#[test]
fn applies_disable_plan_as_empty_managed_files() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxio_core::ProxioConfig::build_disable_plan();

    let results = apply_plan(&plan, &env, None).unwrap();
    assert!(results.iter().all(|item| item.success || item.skipped));

    let shell_path = proxio_adapters::paths::proxio_shell_env_path(dir.path());
    let systemd_path = proxio_adapters::paths::systemd_user_env_path(dir.path());
    assert_eq!(std::fs::read_to_string(shell_path).unwrap(), "");
    assert_eq!(std::fs::read_to_string(systemd_path).unwrap(), "");
}
```

Add this test to `crates/proxio-adapters/tests/command_targets.rs`:

```rust
#[test]
fn applies_disable_plan_to_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let runner = FakeRunner::default();
    let plan = proxio_core::ProxioConfig::build_disable_plan();

    let results = apply_plan(&plan, &env, Some(&runner)).unwrap();
    assert!(results.iter().any(|item| item.target_name == "git" && item.success));

    let commands = runner.commands.lock().unwrap();
    assert!(commands.iter().any(|spec| spec.args.iter().any(|arg| arg == "--unset")));
    assert!(commands.iter().any(|spec| spec.args.iter().any(|arg| arg == "delete")));
}
```

- [ ] **Step 2: Run adapter tests to verify they fail**

Run: `cargo test -p proxio-adapters`
Expected: FAIL because unset behavior is not implemented

- [ ] **Step 3: Update file renderers to omit unset entries**

In `shell_env.rs` and `systemd_user_env.rs`, render only entries whose value is `PlannedEntryValue::Set(_)`. A disable plan should therefore render an empty string.

- [ ] **Step 4: Update command spec builders to support unset operations**

Implement these command forms:

```rust
// git.rs
git config --global --unset http.proxy
git config --global --unset https.proxy

// npm.rs
npm config delete proxy
npm config delete https-proxy

// pnpm.rs
pnpm config delete proxy
pnpm config delete https-proxy
```

Generate them when the planned value is `Unset`.

- [ ] **Step 5: Keep preview and apply orchestration unchanged at the top level, but adapt it to the new plan types**

`crates/proxio-adapters/src/apply.rs` should continue to dispatch by target, but preview summaries and command execution must use the new command specs generated from `Set` and `Unset` values.

- [ ] **Step 6: Run adapter tests to verify they pass**

Run: `cargo test -p proxio-adapters`
Expected: PASS

### Task 4: Add Profile and Disable Commands to the CLI

**Files:**
- Modify: `crates/proxio-cli/src/main.rs`

- [ ] **Step 1: Write the failing CLI behavior tests as shell-driven smoke checks**

Use these command expectations:

```text
PROXIO_HOME=<tmp> cargo run -p proxio -- profile add proxy --http-proxy http://127.0.0.1:7890
Expected: creates a profile named proxy

PROXIO_HOME=<tmp> cargo run -p proxio -- profile use proxy
Expected: updates current_profile only

PROXIO_HOME=<tmp> cargo run -p proxio -- profile current
Expected: prints the active profile and TOML for that profile

PROXIO_HOME=<tmp> cargo run -p proxio -- disable
Expected: clears managed targets while leaving profiles in config.toml
```

- [ ] **Step 2: Run one CLI smoke command to verify it fails before the new subcommands exist**

Run: `PROXIO_HOME="$(mktemp -d)" cargo run -p proxio -- profile list`
Expected: FAIL because `profile` subcommand does not exist yet

- [ ] **Step 3: Extend the CLI command model**

In `crates/proxio-cli/src/main.rs`, replace the command enum with:

```rust
#[derive(Subcommand)]
enum Commands {
    Set {
        #[arg(long)]
        http_proxy: Option<String>,
        #[arg(long)]
        https_proxy: Option<String>,
        #[arg(long)]
        all_proxy: Option<String>,
        #[arg(long, value_delimiter = ',')]
        no_proxy: Vec<String>,
    },
    Show,
    Preview,
    Apply,
    Disable,
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    List,
    Add {
        name: String,
        #[arg(long)]
        http_proxy: Option<String>,
        #[arg(long)]
        https_proxy: Option<String>,
        #[arg(long)]
        all_proxy: Option<String>,
        #[arg(long, value_delimiter = ',')]
        no_proxy: Vec<String>,
    },
    Remove { name: String },
    Use { name: String },
    Current,
}
```

- [ ] **Step 4: Add config helpers for empty config and profile mutation**

In `main.rs`, add:

```rust
fn empty_config() -> ProxioConfig {
    ProxioConfig::new_with_profiles(None, std::iter::empty())
}

fn read_or_default(path: &Path) -> Result<ProxioConfig, String> {
    if path.exists() {
        read_config(path)
    } else {
        Ok(empty_config())
    }
}
```

Then implement inline mutation logic for add, remove, and use using the `profiles` map and `current_profile` field.

- [ ] **Step 5: Route `preview`, `apply`, and `disable` through the new core APIs**

Use:

```rust
let plan = config.build_plan_for_current_profile().map_err(|err| err.to_string())?;
let plan = proxio_core::ProxioConfig::build_disable_plan();
```

for current-profile apply and disable respectively.

- [ ] **Step 6: Run the CLI smoke workflow and verify outputs**

Run:

```bash
tmpdir=$(mktemp -d) && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- profile add proxy --http-proxy http://127.0.0.1:7890 && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- profile list && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- profile use proxy && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- profile current && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- preview && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- disable
```

Expected: all commands succeed and `config.toml` still contains the stored profile after `disable`

### Task 5: Full Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-04-11-proxio-profile-management.md`

- [ ] **Step 1: Run formatting and the full test suite**

Run:

```bash
cargo fmt --all
cargo test
cargo check
```

Expected: PASS

- [ ] **Step 2: Keep the work uncommitted unless the user later asks for a commit**

Do not create a commit during implementation unless the user explicitly requests it.
