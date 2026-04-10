# Proxio Check Diagnose Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `proxio check <url>` so users can run a layered DNS/TCP/TLS/HTTP diagnosis against a target URL using the current active proxy profile.

**Architecture:** Add a new `proxio-diagnose` crate that owns effective proxy selection, layered result models, and diagnosis execution. Keep `proxio-core` responsible for current profile lookup, and keep `proxio-cli` focused on argument parsing and terminal formatting. Use a testable runner abstraction in `proxio-diagnose` so most behavior can be verified without real external networks.

**Tech Stack:** Rust 2024, Cargo workspace, `clap`, `tokio`, `reqwest` with `rustls`, `url`, `serde`

---

### Planned File Structure

**Workspace**
- Modify: `Cargo.toml`

**proxio-diagnose**
- Create: `crates/proxio-diagnose/Cargo.toml`
- Create: `crates/proxio-diagnose/src/lib.rs`
- Create: `crates/proxio-diagnose/src/model.rs`
- Create: `crates/proxio-diagnose/src/proxy.rs`
- Create: `crates/proxio-diagnose/src/runner.rs`
- Create: `crates/proxio-diagnose/src/check.rs`
- Create: `crates/proxio-diagnose/tests/proxy_selection.rs`
- Create: `crates/proxio-diagnose/tests/check_flow.rs`

**proxio-cli**
- Modify: `crates/proxio-cli/Cargo.toml`
- Modify: `crates/proxio-cli/src/main.rs`

### Task 1: Bootstrap `proxio-diagnose` Crate and Result Model

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/proxio-diagnose/Cargo.toml`
- Create: `crates/proxio-diagnose/src/lib.rs`
- Create: `crates/proxio-diagnose/src/model.rs`
- Create: `crates/proxio-diagnose/tests/proxy_selection.rs`

- [ ] **Step 1: Write the failing diagnose model tests**

Create `crates/proxio-diagnose/tests/proxy_selection.rs` with:

```rust
use proxio_core::ProxySettings;
use proxio_diagnose::{EffectiveProxy, TransportMode, select_effective_proxy};

#[test]
fn selects_https_proxy_for_https_urls() {
    let settings = ProxySettings {
        http_proxy: Some("http://127.0.0.1:8080".into()),
        https_proxy: Some("http://127.0.0.1:8443".into()),
        all_proxy: Some("socks5://127.0.0.1:9000".into()),
        no_proxy: vec![],
    };

    let proxy = select_effective_proxy("https://example.com", &settings).unwrap();
    assert_eq!(proxy.mode, TransportMode::Proxied);
    assert_eq!(proxy.value.as_deref(), Some("http://127.0.0.1:8443"));
}

#[test]
fn falls_back_to_all_proxy_when_scheme_specific_proxy_is_missing() {
    let settings = ProxySettings {
        http_proxy: None,
        https_proxy: None,
        all_proxy: Some("socks5://127.0.0.1:9000".into()),
        no_proxy: vec![],
    };

    let proxy = select_effective_proxy("http://example.com", &settings).unwrap();
    assert_eq!(proxy.mode, TransportMode::Proxied);
    assert_eq!(proxy.value.as_deref(), Some("socks5://127.0.0.1:9000"));
}

#[test]
fn uses_direct_mode_when_no_proxy_applies() {
    let settings = ProxySettings {
        http_proxy: None,
        https_proxy: None,
        all_proxy: None,
        no_proxy: vec![],
    };

    let proxy = select_effective_proxy("https://example.com", &settings).unwrap();
    assert_eq!(proxy.mode, TransportMode::Direct);
    assert_eq!(proxy.value, None);
}
```

- [ ] **Step 2: Run the new test target to verify it fails**

Run: `cargo test -p proxio-diagnose --test proxy_selection`
Expected: FAIL because the crate and API do not exist yet

- [ ] **Step 3: Add the workspace member and crate manifest**

Update root `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/proxio-core",
  "crates/proxio-adapters",
  "crates/proxio-cli",
  "crates/proxio-diagnose",
]
```

Create `crates/proxio-diagnose/Cargo.toml`:

```toml
[package]
name = "proxio-diagnose"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
proxio-core = { path = "../proxio-core" }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
tokio = { version = "1", features = ["rt", "net", "time"] }
url.workspace = true
```

- [ ] **Step 4: Add the initial model and proxy-selection API**

Create `crates/proxio-diagnose/src/model.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerStatus {
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Direct,
    Proxied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveProxy {
    pub mode: TransportMode,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerReport {
    pub status: LayerStatus,
    pub summary: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    pub target_url: String,
    pub profile_name: String,
    pub transport: EffectiveProxy,
    pub dns: LayerReport,
    pub tcp: LayerReport,
    pub tls: LayerReport,
    pub http: LayerReport,
    pub conclusion: String,
}
```

Create `crates/proxio-diagnose/src/proxy.rs` with:

```rust
use proxio_core::ProxySettings;
use url::Url;

use crate::model::{EffectiveProxy, TransportMode};

pub fn select_effective_proxy(url: &str, settings: &ProxySettings) -> Result<EffectiveProxy, String> {
    let url = Url::parse(url).map_err(|err| err.to_string())?;
    let value = match url.scheme() {
        "https" => settings
            .https_proxy
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .or_else(|| settings.all_proxy.as_ref().filter(|value| !value.trim().is_empty()).cloned()),
        "http" => settings
            .http_proxy
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .or_else(|| settings.all_proxy.as_ref().filter(|value| !value.trim().is_empty()).cloned()),
        scheme => return Err(format!("unsupported URL scheme: {scheme}")),
    };

    Ok(match value {
        Some(value) => EffectiveProxy {
            mode: TransportMode::Proxied,
            value: Some(value),
        },
        None => EffectiveProxy {
            mode: TransportMode::Direct,
            value: None,
        },
    })
}
```

- [ ] **Step 5: Export the public API and rerun the test target**

Create `crates/proxio-diagnose/src/lib.rs`:

```rust
pub mod check;
pub mod model;
pub mod proxy;
pub mod runner;

pub use model::{CheckReport, EffectiveProxy, LayerReport, LayerStatus, TransportMode};
pub use proxy::select_effective_proxy;
```

Create empty stubs for `check.rs` and `runner.rs`, then run:

Run: `cargo test -p proxio-diagnose --test proxy_selection`
Expected: PASS

### Task 2: Implement Layered Diagnosis Flow with a Testable Runner

**Files:**
- Create: `crates/proxio-diagnose/src/runner.rs`
- Create: `crates/proxio-diagnose/src/check.rs`
- Create: `crates/proxio-diagnose/tests/check_flow.rs`

- [ ] **Step 1: Write failing diagnosis-flow tests**

Create `crates/proxio-diagnose/tests/check_flow.rs` with:

```rust
use proxio_diagnose::{check_url_with_runner, LayerStatus, Runner, RunnerOutcome};

#[derive(Clone)]
struct FakeRunner {
    dns: RunnerOutcome,
    tcp: RunnerOutcome,
    tls: RunnerOutcome,
    http: RunnerOutcome,
}

impl Runner for FakeRunner {
    fn check_dns(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome { self.dns.clone() }
    fn check_tcp(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome { self.tcp.clone() }
    fn check_tls(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome { self.tls.clone() }
    fn check_http(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome { self.http.clone() }
}

#[test]
fn skips_later_layers_after_dns_failure() {
    let report = check_url_with_runner(
        "proxy",
        "https://example.com",
        Some("http://127.0.0.1:7890"),
        &FakeRunner {
            dns: RunnerOutcome::failed("dns failed", "lookup error"),
            tcp: RunnerOutcome::success("tcp ok", ""),
            tls: RunnerOutcome::success("tls ok", ""),
            http: RunnerOutcome::success("http ok", ""),
        },
    )
    .unwrap();

    assert_eq!(report.dns.status, LayerStatus::Failed);
    assert_eq!(report.tcp.status, LayerStatus::Skipped);
    assert_eq!(report.tls.status, LayerStatus::Skipped);
    assert_eq!(report.http.status, LayerStatus::Skipped);
}

#[test]
fn skips_http_after_tls_failure() {
    let report = check_url_with_runner(
        "proxy",
        "https://example.com",
        Some("http://127.0.0.1:7890"),
        &FakeRunner {
            dns: RunnerOutcome::success("dns ok", ""),
            tcp: RunnerOutcome::success("tcp ok", ""),
            tls: RunnerOutcome::failed("tls failed", "certificate error"),
            http: RunnerOutcome::success("http ok", ""),
        },
    )
    .unwrap();

    assert_eq!(report.tls.status, LayerStatus::Failed);
    assert_eq!(report.http.status, LayerStatus::Skipped);
}
```

- [ ] **Step 2: Run the flow tests to verify they fail**

Run: `cargo test -p proxio-diagnose --test check_flow`
Expected: FAIL because the runner abstraction and flow API do not exist yet

- [ ] **Step 3: Implement the runner abstraction**

Create `crates/proxio-diagnose/src/runner.rs`:

```rust
#[derive(Debug, Clone)]
pub struct RunnerOutcome {
    pub success: bool,
    pub summary: String,
    pub detail: String,
}

impl RunnerOutcome {
    pub fn success(summary: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { success: true, summary: summary.into(), detail: detail.into() }
    }

    pub fn failed(summary: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { success: false, summary: summary.into(), detail: detail.into() }
    }
}

pub trait Runner {
    fn check_dns(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
    fn check_tcp(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
    fn check_tls(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
    fn check_http(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
}
```

- [ ] **Step 4: Implement the layered flow API**

Create `crates/proxio-diagnose/src/check.rs` with:

```rust
use url::Url;

use crate::model::{CheckReport, EffectiveProxy, LayerReport, LayerStatus, TransportMode};
use crate::proxy::select_effective_proxy;
use crate::runner::{Runner, RunnerOutcome};

pub fn check_url_with_runner(
    profile_name: &str,
    url: &str,
    effective_proxy: Option<&str>,
    runner: &dyn Runner,
) -> Result<CheckReport, String> {
    let parsed = Url::parse(url).map_err(|err| err.to_string())?;
    let transport = match effective_proxy {
        Some(proxy) => EffectiveProxy { mode: TransportMode::Proxied, value: Some(proxy.to_owned()) },
        None => EffectiveProxy { mode: TransportMode::Direct, value: None },
    };

    let dns = to_layer_report(runner.check_dns(parsed.host_str().unwrap_or_default(), effective_proxy));
    if dns.status == LayerStatus::Failed {
        return Ok(skipped_after(profile_name, url, transport, dns, "DNS resolution failed"));
    }

    let tcp = to_layer_report(runner.check_tcp(parsed.host_str().unwrap_or_default(), effective_proxy));
    if tcp.status == LayerStatus::Failed {
        return Ok(skipped_after_tcp(profile_name, url, transport, dns, tcp, "TCP connection failed"));
    }

    let tls = if parsed.scheme() == "https" {
        to_layer_report(runner.check_tls(parsed.host_str().unwrap_or_default(), effective_proxy))
    } else {
        skipped("TLS not required for HTTP targets")
    };
    if parsed.scheme() == "https" && tls.status == LayerStatus::Failed {
        return Ok(skipped_after_tls(profile_name, url, transport, dns, tcp, tls, "TLS negotiation failed"));
    }

    let http = to_layer_report(runner.check_http(url, effective_proxy));
    let conclusion = if http.status == LayerStatus::Success {
        "HTTP request completed".to_owned()
    } else {
        http.summary.clone()
    };

    Ok(CheckReport { target_url: url.into(), profile_name: profile_name.into(), transport, dns, tcp, tls, http, conclusion })
}
```

Use small helpers in the same file for `to_layer_report`, `skipped`, and the skipped-after constructors.

- [ ] **Step 5: Add a convenience function that combines proxy selection with the runner flow**

In `check.rs`, add:

```rust
pub fn build_check_report(
    profile_name: &str,
    url: &str,
    settings: &proxio_core::ProxySettings,
    runner: &dyn Runner,
) -> Result<CheckReport, String> {
    let effective = select_effective_proxy(url, settings)?;
    check_url_with_runner(profile_name, url, effective.value.as_deref(), runner)
}
```

- [ ] **Step 6: Run the diagnose tests to verify they pass**

Run: `cargo test -p proxio-diagnose`
Expected: PASS

### Task 3: Add a Minimal Real Runner for CLI Use

**Files:**
- Modify: `crates/proxio-diagnose/src/runner.rs`
- Modify: `crates/proxio-diagnose/src/check.rs`

- [ ] **Step 1: Write a small failing real-runner smoke test or compile target expectation**

Use this target expectation:

```text
The crate should compile with a `RealRunner` type implementing `Runner`.
```

- [ ] **Step 2: Run cargo check to verify the real runner does not exist yet**

Run: `cargo check -p proxio-diagnose`
Expected: FAIL if you reference `RealRunner` before implementation

- [ ] **Step 3: Implement `RealRunner` with lightweight layered checks**

Extend `runner.rs` with:

```rust
pub struct RealRunner;

impl Runner for RealRunner {
    fn check_dns(&self, target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        match std::net::ToSocketAddrs::to_socket_addrs(&(target, 443)) {
            Ok(mut addrs) if addrs.next().is_some() => RunnerOutcome::success(format!("resolved {target}"), ""),
            Ok(_) => RunnerOutcome::failed("no socket addresses resolved", ""),
            Err(err) => RunnerOutcome::failed("DNS resolution failed", err.to_string()),
        }
    }

    fn check_tcp(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome {
        let endpoint = proxy.unwrap_or(target);
        RunnerOutcome::success(format!("connection path ready for {endpoint}"), "real TCP probing is minimal in MVP")
    }

    fn check_tls(&self, target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        RunnerOutcome::success(format!("TLS layer prepared for {target}"), "TLS probing is inferred in MVP")
    }

    fn check_http(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome {
        let client = match proxy {
            Some(proxy) => reqwest::blocking::Client::builder().proxy(reqwest::Proxy::all(proxy).unwrap()).build(),
            None => reqwest::blocking::Client::builder().build(),
        };
        match client {
            Ok(client) => match client.get(target).send() {
                Ok(response) => RunnerOutcome::success(format!("received HTTP {}", response.status()), ""),
                Err(err) => RunnerOutcome::failed(classify_http_error(&err), err.to_string()),
            },
            Err(err) => RunnerOutcome::failed("failed to build HTTP client", err.to_string()),
        }
    }
}
```

Also add a small helper in the same file:

```rust
fn classify_http_error(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "HTTP request timed out"
    } else {
        "HTTP request failed"
    }
}
```

Use `reqwest::blocking` to keep CLI integration simple for the MVP.

- [ ] **Step 4: Export `RealRunner` and rerun crate verification**

Run:

```bash
cargo test -p proxio-diagnose
cargo check -p proxio-diagnose
```

Expected: PASS

### Task 4: Add `proxio check <url>` to the CLI

**Files:**
- Modify: `crates/proxio-cli/Cargo.toml`
- Modify: `crates/proxio-cli/src/main.rs`

- [ ] **Step 1: Write the failing CLI smoke expectation**

Use these command expectations:

```text
PROXIO_HOME=<tmp> cargo run -p proxio -- check https://example.com
Expected: either diagnosis output or a clear current-profile/config error, but not “unknown subcommand”
```

- [ ] **Step 2: Run the smoke command to verify `check` is missing**

Run: `PROXIO_HOME="$(mktemp -d)" cargo run -p proxio -- check https://example.com`
Expected: FAIL because `check` subcommand does not exist yet

- [ ] **Step 3: Add the diagnose dependency and CLI command variant**

Update `crates/proxio-cli/Cargo.toml`:

```toml
[dependencies]
clap.workspace = true
proxio-core = { path = "../proxio-core" }
proxio-adapters = { path = "../proxio-adapters" }
proxio-diagnose = { path = "../proxio-diagnose" }
serde.workspace = true
toml.workspace = true
```

Update the CLI command enum in `main.rs`:

```rust
    Check {
        url: String,
    },
```

- [ ] **Step 4: Route `check` through current-profile resolution and `RealRunner`**

In `run()`, add:

```rust
        Commands::Check { url } => {
            let config = read_or_default(&path)?;
            let (profile_name, settings) = config.current_profile().map_err(|err| err.to_string())?;
            let report = proxio_diagnose::check::build_check_report(
                profile_name,
                &url,
                settings,
                &proxio_diagnose::runner::RealRunner,
            )?;
            print_check_report(&report);
        }
```

- [ ] **Step 5: Add a formatter for the diagnosis output**

In `main.rs`, add:

```rust
fn print_check_report(report: &proxio_diagnose::CheckReport) {
    let mode = match &report.transport.value {
        Some(proxy) => format!("proxied via {proxy}"),
        None => "direct".to_owned(),
    };

    println!("Target: {}", report.target_url);
    println!("Profile: {}", report.profile_name);
    println!("Mode: {}", mode);
    println!();
    println!("DNS  : {} - {}", format_status(report.dns.status), report.dns.summary);
    println!("TCP  : {} - {}", format_status(report.tcp.status), report.tcp.summary);
    println!("TLS  : {} - {}", format_status(report.tls.status), report.tls.summary);
    println!("HTTP : {} - {}", format_status(report.http.status), report.http.summary);
    println!();
    println!("Conclusion: {}", report.conclusion);
}

fn format_status(status: proxio_diagnose::LayerStatus) -> &'static str {
    match status {
        proxio_diagnose::LayerStatus::Success => "success",
        proxio_diagnose::LayerStatus::Failed => "failed",
        proxio_diagnose::LayerStatus::Skipped => "skipped",
    }
}
```

- [ ] **Step 6: Run a CLI smoke check with a configured profile**

Run:

```bash
tmpdir=$(mktemp -d) && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- profile add proxy --http-proxy http://127.0.0.1:7890 && \
PROXIO_HOME="$tmpdir" cargo run -p proxio -- check https://example.com
```

Expected: command runs and prints layered output beginning with `Target:`, `Profile:`, and `Mode:`

### Task 5: Final Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-04-11-proxio-check-diagnose.md`

- [ ] **Step 1: Run formatting and the full workspace verification**

Run:

```bash
cargo fmt --all
cargo test
cargo check
```

Expected: PASS

- [ ] **Step 2: Leave changes uncommitted unless the user later requests a commit**

Do not create a commit during implementation unless the user explicitly asks for one.
