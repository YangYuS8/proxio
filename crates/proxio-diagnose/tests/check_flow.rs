use proxio_diagnose::{LayerStatus, Runner, RunnerOutcome, check_url_with_runner};

#[derive(Clone)]
struct FakeRunner {
    dns: RunnerOutcome,
    tcp: RunnerOutcome,
    tls: RunnerOutcome,
    http: RunnerOutcome,
}

impl Runner for FakeRunner {
    fn check_dns(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        self.dns.clone()
    }

    fn check_tcp(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        self.tcp.clone()
    }

    fn check_tls(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        self.tls.clone()
    }

    fn check_http(&self, _target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        self.http.clone()
    }
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
