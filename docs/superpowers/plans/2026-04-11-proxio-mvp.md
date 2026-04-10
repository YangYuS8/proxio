# Proxio MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first usable Proxio MVP as a Rust Cargo workspace with a CLI that stores proxy settings, previews changes, and applies them to shell env, systemd user environment, Git, npm, and pnpm.

**Architecture:** Use `proxio-core` for config, validation, and target-independent apply-plan generation; use `proxio-adapters` for concrete file and command side effects; use `proxio-cli` for argument parsing and user-facing output. Keep preview and apply aligned by driving both from the same `ApplyPlan`.

**Tech Stack:** Rust 2024, Cargo workspace, `clap`, `serde`, `toml`, `url`, `tempfile`

---

### Planned File Structure

**Workspace files**
- Create: `Cargo.toml`
- Create: `Cargo.lock`

**proxio-core**
- Create: `crates/proxio-core/Cargo.toml`
- Create: `crates/proxio-core/src/lib.rs`
- Create: `crates/proxio-core/src/config.rs`
- Create: `crates/proxio-core/src/plan.rs`
- Create: `crates/proxio-core/src/validate.rs`

**proxio-adapters**
- Create: `crates/proxio-adapters/Cargo.toml`
- Create: `crates/proxio-adapters/src/lib.rs`
- Create: `crates/proxio-adapters/src/paths.rs`
- Create: `crates/proxio-adapters/src/file_ops.rs`
- Create: `crates/proxio-adapters/src/command_runner.rs`
- Create: `crates/proxio-adapters/src/shell_env.rs`
- Create: `crates/proxio-adapters/src/systemd_user_env.rs`
- Create: `crates/proxio-adapters/src/git.rs`
- Create: `crates/proxio-adapters/src/npm.rs`
- Create: `crates/proxio-adapters/src/pnpm.rs`
- Create: `crates/proxio-adapters/src/apply.rs`

**proxio-cli**
- Create: `crates/proxio-cli/Cargo.toml`
- Create: `crates/proxio-cli/src/main.rs`

**Integration tests**
- Create: `crates/proxio-core/tests/config_and_plan.rs`
- Create: `crates/proxio-adapters/tests/file_targets.rs`
- Create: `crates/proxio-adapters/tests/command_targets.rs`

### Task 1: Bootstrap Cargo Workspace

**Files:**
- Create: `Cargo.toml`
- Create: `crates/proxio-core/Cargo.toml`
- Create: `crates/proxio-core/src/lib.rs`
- Create: `crates/proxio-adapters/Cargo.toml`
- Create: `crates/proxio-adapters/src/lib.rs`
- Create: `crates/proxio-cli/Cargo.toml`
- Create: `crates/proxio-cli/src/main.rs`

- [ ] **Step 1: Write the failing workspace smoke test via cargo check expectation**

Use this root manifest skeleton:

```toml
[workspace]
members = [
  "crates/proxio-core",
  "crates/proxio-adapters",
  "crates/proxio-cli",
]
resolver = "2"

[workspace.package]
edition = "2024"
license = "MIT"
version = "0.1.0"

[workspace.dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
tempfile = "3"
toml = "0.8"
url = "2.5"
```

- [ ] **Step 2: Run workspace check to verify it fails before crate files exist**

Run: `cargo check`
Expected: FAIL with missing crate manifests or source files

- [ ] **Step 3: Add minimal crate manifests and entry points**

Create these crate manifests:

```toml
# crates/proxio-core/Cargo.toml
[package]
name = "proxio-core"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
serde.workspace = true
toml.workspace = true
url.workspace = true
```

```toml
# crates/proxio-adapters/Cargo.toml
[package]
name = "proxio-adapters"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
proxio-core = { path = "../proxio-core" }
tempfile.workspace = true
```

```toml
# crates/proxio-cli/Cargo.toml
[package]
name = "proxio"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
clap.workspace = true
proxio-core = { path = "../proxio-core" }
proxio-adapters = { path = "../proxio-adapters" }
serde.workspace = true
toml.workspace = true
```

Create these minimal sources:

```rust
// crates/proxio-core/src/lib.rs
pub mod config;
pub mod plan;
pub mod validate;
```

```rust
// crates/proxio-adapters/src/lib.rs
pub mod apply;
pub mod command_runner;
pub mod file_ops;
pub mod git;
pub mod npm;
pub mod paths;
pub mod pnpm;
pub mod shell_env;
pub mod systemd_user_env;
```

```rust
// crates/proxio-cli/src/main.rs
fn main() {
    println!("proxio");
}
```

- [ ] **Step 4: Add stub source files so the workspace compiles structurally**

Create empty-but-valid module files with one public placeholder type per file, for example:

```rust
// crates/proxio-core/src/config.rs
pub struct Stub;
```

- [ ] **Step 5: Run workspace check to verify bootstrap passes**

Run: `cargo check`
Expected: PASS

### Task 2: Implement proxio-core Config, Validation, and ApplyPlan

**Files:**
- Modify: `crates/proxio-core/src/lib.rs`
- Replace: `crates/proxio-core/src/config.rs`
- Replace: `crates/proxio-core/src/plan.rs`
- Replace: `crates/proxio-core/src/validate.rs`
- Create: `crates/proxio-core/tests/config_and_plan.rs`

- [ ] **Step 1: Write the failing core tests**

Create these tests:

```rust
use proxio_core::config::{ProxioConfig, ProxySettings};
use proxio_core::plan::TargetKind;

#[test]
fn normalizes_no_proxy_and_builds_all_targets() {
    let config = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: Some("http://127.0.0.1:7890".into()),
            all_proxy: Some("socks5://127.0.0.1:7891".into()),
            no_proxy: vec![" localhost ".into(), "127.0.0.1".into(), "localhost".into()],
        },
    };

    let plan = config.build_plan().expect("plan should build");
    assert_eq!(config.proxy.normalized_no_proxy(), vec!["127.0.0.1", "localhost"]);
    assert_eq!(plan.operations.len(), 5);
    assert!(plan.operations.iter().any(|op| op.target == TargetKind::ShellEnv));
    assert!(plan.operations.iter().any(|op| op.target == TargetKind::SystemdUserEnv));
    assert!(plan.operations.iter().any(|op| op.target == TargetKind::Git));
    assert!(plan.operations.iter().any(|op| op.target == TargetKind::Npm));
    assert!(plan.operations.iter().any(|op| op.target == TargetKind::Pnpm));
}

#[test]
fn rejects_invalid_proxy_url() {
    let config = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("not-a-url".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: vec![],
        },
    };

    let error = config.build_plan().expect_err("invalid url should fail");
    assert!(error.to_string().contains("http_proxy"));
}
```

- [ ] **Step 2: Run core tests to verify they fail**

Run: `cargo test -p proxio-core`
Expected: FAIL with missing types and methods

- [ ] **Step 3: Implement persisted config and proxy settings model**

Use this shape:

```rust
// crates/proxio-core/src/config.rs
use serde::{Deserialize, Serialize};

use crate::plan::ApplyPlan;
use crate::validate::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxioConfig {
    pub proxy: ProxySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxySettings {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub all_proxy: Option<String>,
    #[serde(default)]
    pub no_proxy: Vec<String>,
}

impl ProxySettings {
    pub fn normalized_no_proxy(&self) -> Vec<String> {
        let mut values: Vec<String> = self
            .no_proxy
            .iter()
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        values.sort();
        values.dedup();
        values
    }
}

impl ProxioConfig {
    pub fn build_plan(&self) -> Result<ApplyPlan, ValidationError> {
        crate::validate::validate_proxy_settings(&self.proxy)?;
        Ok(ApplyPlan::from_settings(&self.proxy))
    }
}
```

- [ ] **Step 4: Implement target model and apply-plan generation**

Use this shape:

```rust
// crates/proxio-core/src/plan.rs
use crate::config::ProxySettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    ShellEnv,
    SystemdUserEnv,
    Git,
    Npm,
    Pnpm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedOperation {
    pub target: TargetKind,
    pub entries: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPlan {
    pub operations: Vec<PlannedOperation>,
}

impl ApplyPlan {
    pub fn from_settings(settings: &ProxySettings) -> Self {
        let mut entries = Vec::new();
        if let Some(value) = settings.http_proxy.as_ref().filter(|v| !v.is_empty()) {
            entries.push(("http_proxy".into(), value.clone()));
        }
        if let Some(value) = settings.https_proxy.as_ref().filter(|v| !v.is_empty()) {
            entries.push(("https_proxy".into(), value.clone()));
        }
        if let Some(value) = settings.all_proxy.as_ref().filter(|v| !v.is_empty()) {
            entries.push(("all_proxy".into(), value.clone()));
        }
        let no_proxy = settings.normalized_no_proxy().join(",");
        if !no_proxy.is_empty() {
            entries.push(("no_proxy".into(), no_proxy));
        }

        let targets = [
            TargetKind::ShellEnv,
            TargetKind::SystemdUserEnv,
            TargetKind::Git,
            TargetKind::Npm,
            TargetKind::Pnpm,
        ];

        let operations = targets
            .into_iter()
            .map(|target| PlannedOperation {
                target,
                entries: entries.clone(),
            })
            .collect();

        Self { operations }
    }
}
```

- [ ] **Step 5: Implement validation errors and URL checks**

Use this shape:

```rust
// crates/proxio-core/src/validate.rs
use std::error::Error;
use std::fmt::{Display, Formatter};

use url::Url;

use crate::config::ProxySettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl Error for ValidationError {}

pub fn validate_proxy_settings(settings: &ProxySettings) -> Result<(), ValidationError> {
    validate_url("http_proxy", settings.http_proxy.as_deref())?;
    validate_url("https_proxy", settings.https_proxy.as_deref())?;
    validate_url("all_proxy", settings.all_proxy.as_deref())?;
    Ok(())
}

fn validate_url(field: &'static str, value: Option<&str>) -> Result<(), ValidationError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    Url::parse(value).map_err(|error| ValidationError {
        field,
        message: error.to_string(),
    })?;

    Ok(())
}
```

- [ ] **Step 6: Export the public API from `lib.rs`**

Use this shape:

```rust
pub mod config;
pub mod plan;
pub mod validate;

pub use config::{ProxioConfig, ProxySettings};
pub use plan::{ApplyPlan, PlannedOperation, TargetKind};
pub use validate::ValidationError;
```

- [ ] **Step 7: Run core tests to verify they pass**

Run: `cargo test -p proxio-core`
Expected: PASS

### Task 3: Implement proxio-adapters Preview and Apply Layers

**Files:**
- Modify: `crates/proxio-adapters/Cargo.toml`
- Replace: `crates/proxio-adapters/src/lib.rs`
- Replace: `crates/proxio-adapters/src/paths.rs`
- Replace: `crates/proxio-adapters/src/file_ops.rs`
- Replace: `crates/proxio-adapters/src/command_runner.rs`
- Replace: `crates/proxio-adapters/src/shell_env.rs`
- Replace: `crates/proxio-adapters/src/systemd_user_env.rs`
- Replace: `crates/proxio-adapters/src/git.rs`
- Replace: `crates/proxio-adapters/src/npm.rs`
- Replace: `crates/proxio-adapters/src/pnpm.rs`
- Replace: `crates/proxio-adapters/src/apply.rs`
- Create: `crates/proxio-adapters/tests/file_targets.rs`
- Create: `crates/proxio-adapters/tests/command_targets.rs`

- [ ] **Step 1: Write failing adapter tests for file and command targets**

Create these tests:

```rust
// crates/proxio-adapters/tests/file_targets.rs
use proxio_adapters::apply::{apply_plan, preview_plan, ApplyEnvironment};
use proxio_core::config::{ProxioConfig, ProxySettings};

#[test]
fn previews_shell_and_systemd_files() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: vec!["localhost".into()],
        },
    }
    .build_plan()
    .unwrap();

    let preview = preview_plan(&plan, &env, None).unwrap();
    assert!(preview.iter().any(|item| item.target_name == "shell_env"));
    assert!(preview.iter().any(|item| item.target_name == "systemd_user_env"));
}

#[test]
fn applies_shell_and_systemd_files() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: Some("http://127.0.0.1:7890".into()),
            all_proxy: None,
            no_proxy: vec![],
        },
    }
    .build_plan()
    .unwrap();

    let results = apply_plan(&plan, &env, None).unwrap();
    assert!(results.iter().any(|item| item.target_name == "shell_env" && item.success));
    assert!(results.iter().any(|item| item.target_name == "systemd_user_env" && item.success));
}
```

```rust
// crates/proxio-adapters/tests/command_targets.rs
use proxio_adapters::apply::{apply_plan, preview_plan, ApplyEnvironment};
use proxio_adapters::command_runner::{CommandRunner, CommandSpec, CommandStatus};
use proxio_core::config::{ProxioConfig, ProxySettings};

#[derive(Default)]
struct FakeRunner {
    commands: std::sync::Mutex<Vec<CommandSpec>>,
}

impl CommandRunner for FakeRunner {
    fn command_exists(&self, _program: &str) -> bool {
        true
    }

    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String> {
        self.commands.lock().unwrap().push(spec.clone());
        Ok(CommandStatus { success: true, stderr: String::new() })
    }
}

#[test]
fn previews_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: vec![],
        },
    }
    .build_plan()
    .unwrap();

    let preview = preview_plan(&plan, &env, Some(&FakeRunner::default())).unwrap();
    assert!(preview.iter().any(|item| item.target_name == "git"));
    assert!(preview.iter().any(|item| item.target_name == "npm"));
    assert!(preview.iter().any(|item| item.target_name == "pnpm"));
}

#[test]
fn applies_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let runner = FakeRunner::default();
    let plan = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: Some("http://127.0.0.1:7890".into()),
            all_proxy: None,
            no_proxy: vec![],
        },
    }
    .build_plan()
    .unwrap();

    let results = apply_plan(&plan, &env, Some(&runner)).unwrap();
    assert!(results.iter().any(|item| item.target_name == "git"));
    assert!(results.iter().any(|item| item.target_name == "npm"));
    assert!(results.iter().any(|item| item.target_name == "pnpm"));
}
```

- [ ] **Step 2: Run adapter tests to verify they fail**

Run: `cargo test -p proxio-adapters`
Expected: FAIL with missing adapter API

- [ ] **Step 3: Add dependencies and shared adapter types**

Update `crates/proxio-adapters/Cargo.toml` to include:

```toml
[dependencies]
proxio-core = { path = "../proxio-core" }
tempfile.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

Define public result types in `src/apply.rs`:

```rust
use std::path::{Path, PathBuf};

use proxio_core::{ApplyPlan, PlannedOperation, TargetKind};

use crate::command_runner::{CommandRunner, RealCommandRunner};

#[derive(Debug, Clone)]
pub struct ApplyEnvironment {
    pub root: PathBuf,
}

impl ApplyEnvironment {
    pub fn for_root(root: &Path) -> Self {
        Self { root: root.to_path_buf() }
    }
}

#[derive(Debug, Clone)]
pub struct PreviewItem {
    pub target_name: &'static str,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct ApplyResultItem {
    pub target_name: &'static str,
    pub success: bool,
    pub skipped: bool,
    pub message: String,
}
```

- [ ] **Step 4: Implement centralized paths and atomic file helper**

Use this shape:

```rust
// crates/proxio-adapters/src/paths.rs
use std::path::{Path, PathBuf};

pub fn proxio_config_dir(root: &Path) -> PathBuf {
    root.join(".config/proxio")
}

pub fn proxio_shell_env_path(root: &Path) -> PathBuf {
    proxio_config_dir(root).join("env/proxy.env")
}

pub fn systemd_user_env_path(root: &Path) -> PathBuf {
    root.join(".config/environment.d/proxio-proxy.conf")
}
```

```rust
// crates/proxio-adapters/src/file_ops.rs
use std::fs;
use std::path::Path;

pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "missing parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content).map_err(|err| err.to_string())?;
    fs::rename(&temp_path, path).map_err(|err| err.to_string())?;
    Ok(())
}
```

- [ ] **Step 5: Implement command runner abstraction**

Use this shape:

```rust
// crates/proxio-adapters/src/command_runner.rs
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandStatus {
    pub success: bool,
    pub stderr: String,
}

pub trait CommandRunner {
    fn command_exists(&self, program: &str) -> bool;
    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String>;
}

pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn command_exists(&self, program: &str) -> bool {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} >/dev/null 2>&1", program))
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String> {
        let output = std::process::Command::new(&spec.program)
            .args(&spec.args)
            .output()
            .map_err(|err| err.to_string())?;

        Ok(CommandStatus {
            success: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
```

- [ ] **Step 6: Implement file target rendering helpers**

Use these shapes:

```rust
// crates/proxio-adapters/src/shell_env.rs
use proxio_core::PlannedOperation;

pub fn render(operation: &PlannedOperation) -> String {
    operation
        .entries
        .iter()
        .map(|(key, value)| format!("export {}={:?}", key, value))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
```

```rust
// crates/proxio-adapters/src/systemd_user_env.rs
use proxio_core::PlannedOperation;

pub fn render(operation: &PlannedOperation) -> String {
    operation
        .entries
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
```

- [ ] **Step 7: Implement command-target spec builders**

Use this shape:

```rust
// crates/proxio-adapters/src/git.rs
use proxio_adapters::command_runner::CommandSpec;
use proxio_core::PlannedOperation;

pub fn specs(operation: &PlannedOperation) -> Vec<CommandSpec> {
    operation
        .entries
        .iter()
        .filter_map(|(key, value)| match key.as_str() {
            "http_proxy" => Some(CommandSpec { program: "git".into(), args: vec!["config".into(), "--global".into(), "http.proxy".into(), value.clone()] }),
            "https_proxy" => Some(CommandSpec { program: "git".into(), args: vec!["config".into(), "--global".into(), "https.proxy".into(), value.clone()] }),
            _ => None,
        })
        .collect()
}
```

Implement `npm.rs` and `pnpm.rs` with equivalent `specs()` functions using `npm config set` and `pnpm config set` for `proxy` and `https-proxy`.

- [ ] **Step 8: Implement preview and apply orchestration**

Use this structure in `src/apply.rs`:

```rust
pub fn preview_plan(
    plan: &ApplyPlan,
    env: &ApplyEnvironment,
    runner: Option<&dyn CommandRunner>,
) -> Result<Vec<PreviewItem>, String> {
    let runner = runner.unwrap_or(&RealCommandRunner);
    let mut items = Vec::new();

    for operation in &plan.operations {
        match operation.target {
            TargetKind::ShellEnv => {
                let path = crate::paths::proxio_shell_env_path(&env.root);
                items.push(PreviewItem { target_name: "shell_env", summary: format!("write {}\n{}", path.display(), crate::shell_env::render(operation)) });
            }
            TargetKind::SystemdUserEnv => {
                let path = crate::paths::systemd_user_env_path(&env.root);
                items.push(PreviewItem { target_name: "systemd_user_env", summary: format!("write {}\n{}", path.display(), crate::systemd_user_env::render(operation)) });
            }
            TargetKind::Git => {
                items.push(PreviewItem { target_name: "git", summary: format!("{} command(s)", crate::git::specs(operation).len()) });
            }
            TargetKind::Npm => {
                let exists = runner.command_exists("npm");
                items.push(PreviewItem { target_name: "npm", summary: if exists { "will run npm config set".into() } else { "skipped: npm not found".into() } });
            }
            TargetKind::Pnpm => {
                let exists = runner.command_exists("pnpm");
                items.push(PreviewItem { target_name: "pnpm", summary: if exists { "will run pnpm config set".into() } else { "skipped: pnpm not found".into() } });
            }
        }
    }

    Ok(items)
}
```

Implement `apply_plan()` to mirror the same target dispatch and return `ApplyResultItem` values instead of preview text.

- [ ] **Step 9: Export the adapter API from `lib.rs`**

Use this shape:

```rust
pub mod apply;
pub mod command_runner;
pub mod file_ops;
pub mod git;
pub mod npm;
pub mod paths;
pub mod pnpm;
pub mod shell_env;
pub mod systemd_user_env;

pub use apply::{apply_plan, preview_plan, ApplyEnvironment, ApplyResultItem, PreviewItem};
pub use command_runner::{CommandRunner, CommandSpec, CommandStatus, RealCommandRunner};
```

- [ ] **Step 10: Run adapter tests to verify they pass**

Run: `cargo test -p proxio-adapters`
Expected: PASS

### Task 4: Implement proxio CLI Commands and Config Persistence

**Files:**
- Modify: `crates/proxio-cli/Cargo.toml`
- Replace: `crates/proxio-cli/src/main.rs`

- [ ] **Step 1: Write the failing CLI smoke checks**

Use these manual expectations as the first target:

```text
cargo run -p proxio -- show
Expected: reports missing config or empty config cleanly

cargo run -p proxio -- set --http-proxy http://127.0.0.1:7890 --https-proxy http://127.0.0.1:7890 --no-proxy localhost,127.0.0.1
Expected: writes ~/.config/proxio/config.toml or the path from PROXIO_HOME

cargo run -p proxio -- preview
Expected: prints shell_env, systemd_user_env, git, npm, pnpm preview lines

cargo run -p proxio -- apply
Expected: prints per-target success/skipped/failure summary
```

- [ ] **Step 2: Run the CLI commands to verify the current stub is insufficient**

Run: `cargo run -p proxio -- show`
Expected: FAIL or incorrect output because CLI is still a stub

- [ ] **Step 3: Add CLI dependencies and command model**

Update `crates/proxio-cli/Cargo.toml` dependencies to include:

```toml
[dependencies]
clap.workspace = true
proxio-core = { path = "../proxio-core" }
proxio-adapters = { path = "../proxio-adapters" }
serde.workspace = true
toml.workspace = true
```

Use this CLI model:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "proxio")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
}
```

- [ ] **Step 4: Implement config path resolution and persistence helpers**

Use this shape inside `main.rs`:

```rust
fn proxio_root() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("PROXIO_HOME") {
        return std::path::PathBuf::from(path);
    }

    let home = std::env::var("HOME").expect("HOME must be set");
    std::path::PathBuf::from(home)
}

fn config_path(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".config/proxio/config.toml")
}
```

Implement `read_config()` and `write_config()` around those helpers using `toml::from_str` and `toml::to_string_pretty`.

- [ ] **Step 5: Implement `set` and `show`**

Use this logic:

```rust
match cli.command {
    Commands::Set { http_proxy, https_proxy, all_proxy, no_proxy } => {
        let config = proxio_core::ProxioConfig {
            proxy: proxio_core::ProxySettings { http_proxy, https_proxy, all_proxy, no_proxy },
        };
        write_config(&config_path(&root), &config)?;
        println!("saved {}", config_path(&root).display());
    }
    Commands::Show => {
        let config = read_config(&config_path(&root))?;
        println!("{}", toml::to_string_pretty(&config)?);
    }
    _ => {}
}
```

- [ ] **Step 6: Implement `preview` and `apply`**

Use this logic:

```rust
Commands::Preview => {
    let config = read_config(&config_path(&root))?;
    let plan = config.build_plan()?;
    let env = proxio_adapters::ApplyEnvironment::for_root(&root);
    for item in proxio_adapters::preview_plan(&plan, &env, None)? {
        println!("[{}] {}", item.target_name, item.summary);
    }
}
Commands::Apply => {
    let config = read_config(&config_path(&root))?;
    let plan = config.build_plan()?;
    let env = proxio_adapters::ApplyEnvironment::for_root(&root);
    for item in proxio_adapters::apply_plan(&plan, &env, None)? {
        println!("[{}] {}", item.target_name, item.message);
    }
}
```

- [ ] **Step 7: Run the CLI smoke checks to verify behavior**

Run:

```bash
PROXIO_HOME="$(mktemp -d)" cargo run -p proxio -- set --http-proxy http://127.0.0.1:7890 --https-proxy http://127.0.0.1:7890 --no-proxy localhost,127.0.0.1
PROXIO_HOME="<same-temp-dir>" cargo run -p proxio -- show
PROXIO_HOME="<same-temp-dir>" cargo run -p proxio -- preview
PROXIO_HOME="<same-temp-dir>" cargo run -p proxio -- apply
```

Expected: all commands succeed and print per-target output

### Task 5: Final Verification And Plan Cleanup

**Files:**
- Modify: `docs/superpowers/plans/2026-04-11-proxio-mvp.md`

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: PASS

- [ ] **Step 2: Run cargo fmt and cargo check**

Run:

```bash
cargo fmt --all
cargo check
```

Expected: PASS

- [ ] **Step 3: Mark completed plan steps in this file while executing**

Convert completed checklist items from `- [ ]` to `- [x]` as implementation proceeds.

- [ ] **Step 4: Do not create a git commit unless the user explicitly asks for one**

Repository policy in this session requires leaving changes uncommitted unless the user requests a commit.
