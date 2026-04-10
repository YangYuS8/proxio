## Proxio Check Diagnose Design

### Goal

Add a compact CLI network diagnosis command, `proxio check <url>`, focused on the most common real-world failure mode: a user applies proxy settings and then cannot access a target website. The command should report where the request path fails across DNS, TCP, TLS, and HTTP layers.

### Non-Goals

- No GUI integration in this phase
- No background monitoring or repeated checks
- No auto-repair or config mutation
- No multi-target batch diagnosis
- No automatic scene detection or profile switching
- No attempt to fully diagnose every proxy protocol edge case in the first version

### User Value

The current Proxio MVP can manage and switch proxy profiles, but it still leaves a crucial question unanswered: why does a given URL fail after a profile is applied? This phase gives users a single short command that connects the active profile to a concrete target URL and shows which layer is failing in a way they can act on.

### Technical Impact

This change introduces a new crate, `proxio-diagnose`, to hold diagnosis logic and result models. `proxio-cli` gains `check <url>`. `proxio-core` continues to own current-profile resolution and proxy selection inputs, while `proxio-diagnose` consumes the active proxy settings to produce layered network results.

### Risks

- Proxy behavior can differ by URL scheme and environment in ways that are hard to infer perfectly
- First-version classification may sometimes identify the failing layer correctly but still simplify the deeper root cause
- Real network behavior is hard to test reproducibly without local mocks or fakes
- A naive implementation could hide whether the command used a proxy or connected directly

### Recommended Approach

Build a focused, layered diagnosis pipeline around a single URL input.

`proxio-cli` should parse the URL and active profile, then call `proxio-diagnose`. `proxio-diagnose` should select the effective proxy for the URL scheme, run DNS, TCP, TLS, and HTTP checks in order, and return a structured result model. The CLI should format that result clearly for terminal use.

Alternative approaches were considered:

- A generic health-check command without URL input: rejected because users care about a specific failing target
- A direct `diagnose current` command: rejected for MVP because it is less concrete and less useful during failure
- Full live packet or system-level diagnostics: rejected because it is too heavy for the first usable diagnosis feature

### Scope

This phase adds:

- a new `proxio-diagnose` crate
- `proxio check <url>`
- current-profile-aware proxy selection
- layered DNS, TCP, TLS, and HTTP reporting

This phase does not add:

- GUI diagnosis screens
- historical storage of diagnosis results
- profile-aware automated recommendations
- diagnostics for non-HTTP target types

### Command Design

The new user-facing command is:

```bash
proxio check <url>
```

Examples:

```bash
proxio check https://example.com
proxio check http://intranet.local
```

The command should:

1. Read the current active profile from Proxio config
2. Resolve which proxy variables apply to the given URL scheme
3. Run layered checks in order
4. Print a readable diagnosis summary

### Architecture

#### proxio-core

Responsibilities remain unchanged except for providing:

- current profile name
- current profile proxy settings

`proxio-core` should not implement network operations.

#### proxio-diagnose

Responsibilities:

- choose the effective proxy for a target URL
- model layered diagnosis results
- execute DNS, TCP, TLS, and HTTP checks
- classify errors into actionable summaries

This crate should own the core diagnosis flow rather than embedding logic directly inside the CLI.

#### proxio-cli

Responsibilities:

- parse `check <url>`
- load the current profile through `proxio-core`
- call `proxio-diagnose`
- print layered results and overall conclusion

### Effective Proxy Selection

For the first version, proxy selection should follow these rules:

- for `https://...`
  - prefer `https_proxy`
  - fall back to `all_proxy`
- for `http://...`
  - prefer `http_proxy`
  - fall back to `all_proxy`
- if neither applies
  - treat the request as direct

The command output must explicitly say whether the diagnosis used a proxy and which profile produced that setting.

### Diagnosis Flow

Given a URL and the effective proxy decision, the pipeline should run these layers:

1. **DNS**
   - resolve the target hostname
   - if using a proxy, also resolve the proxy host when relevant
2. **TCP**
   - attempt to connect either to the proxy endpoint or directly to the target endpoint
3. **TLS**
   - for HTTPS targets, attempt a TLS handshake at the relevant layer
   - if the proxy path requires CONNECT, the result should still make clear whether the TLS failure happened after proxy connection succeeded
4. **HTTP**
   - perform a real HTTP request and capture response classification

The implementation should stop or mark later layers as skipped when earlier layers make them impossible.

### Result Model

`proxio-diagnose` should expose a structured result with:

- target URL
- active profile name
- transport mode
  - direct
  - proxied
- effective proxy string, if any
- layered results:
  - `dns`
  - `tcp`
  - `tls`
  - `http`
- final conclusion

Each layer should include at least:

- `status`
  - `success`
  - `failed`
  - `skipped`
- `summary`
  - a short, user-readable explanation
- `detail`
  - the main raw error or supporting context

### Output Expectations

The terminal output should be concise but explicit. A typical output shape is:

```text
Target: https://example.com
Profile: proxy
Mode: proxied via http://127.0.0.1:7890

DNS  : success - resolved example.com
TCP  : success - connected to proxy 127.0.0.1:7890
TLS  : failed  - handshake failed
HTTP : skipped - TLS did not complete

Conclusion: proxy reachable, TLS negotiation failed after proxy connection
```

The first version should prioritize clarity over machine-readable output formats.

### Failure Classification

The first version should classify at least these cases:

- DNS failure
  - target host cannot be resolved
- TCP failure
  - cannot connect to proxy
  - cannot connect directly to target
- TLS failure
  - handshake failure
  - certificate validation error
  - likely proxy protocol mismatch or CONNECT failure context
- HTTP failure
  - timeout
  - proxy returned `407 Proxy Authentication Required`
  - target returned `4xx` or `5xx`

The implementation does not need perfect root-cause inference for every edge case, but the reported layer and summary must remain credible and useful.

### Boundary Conditions

- If no current profile exists, `proxio check <url>` should fail with a clear config error
- If the current profile is direct or otherwise has no effective proxy for the URL, diagnosis should continue as direct mode
- If the URL scheme is unsupported, the command should fail early with a clear error
- If DNS fails, later layers should be marked `skipped`
- If TCP fails, TLS and HTTP should be marked `skipped`
- If TLS fails for an HTTPS target, HTTP should be marked `skipped`

### Dependencies

The design expects `proxio-diagnose` to use Rust-native networking with existing project preferences:

- `tokio` for async runtime support if needed
- `reqwest` with `rustls` for HTTP and higher-level behavior
- standard library resolution and socket primitives where practical

The first implementation should stay lightweight and avoid bringing in unnecessary networking dependencies beyond what is required for the four-layer checks.

### Testing Strategy

#### proxio-diagnose tests

- URL parsing and scheme handling
- effective proxy selection for `http`, `https`, and fallback to `all_proxy`
- layered result model construction
- classification mapping for common error types

#### proxio-cli tests

- `check <url>` argument parsing
- clear error when current profile is missing
- formatting of layered output for direct and proxied runs

#### Integration strategy

- prefer local mock servers or fake diagnostic executors over real internet endpoints
- avoid depending on external network availability in CI-style verification
- keep most tests focused on pure logic and deterministic classification

### Deliverables

The implementation should produce:

- `proxio-diagnose` crate
- `proxio check <url>` command
- current-profile-aware effective proxy selection
- layered DNS, TCP, TLS, and HTTP diagnosis results
- tests for proxy selection, result modeling, and CLI formatting
