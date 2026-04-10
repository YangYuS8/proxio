use proxio_core::ProxySettings;
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
    let host = parsed.host_str().unwrap_or_default();
    let transport = match effective_proxy {
        Some(proxy) => EffectiveProxy {
            mode: TransportMode::Proxied,
            value: Some(proxy.to_owned()),
        },
        None => EffectiveProxy {
            mode: TransportMode::Direct,
            value: None,
        },
    };

    let dns = to_layer_report(runner.check_dns(host, effective_proxy));
    if dns.status == LayerStatus::Failed {
        return Ok(build_skipped_after_dns(
            profile_name,
            url,
            transport,
            dns,
            "DNS resolution failed",
        ));
    }

    let tcp = to_layer_report(runner.check_tcp(host, effective_proxy));
    if tcp.status == LayerStatus::Failed {
        return Ok(build_skipped_after_tcp(
            profile_name,
            url,
            transport,
            dns,
            tcp,
            "TCP connection failed",
        ));
    }

    let tls = if parsed.scheme() == "https" {
        to_layer_report(runner.check_tls(host, effective_proxy))
    } else {
        skipped("TLS not required for HTTP targets")
    };
    if parsed.scheme() == "https" && tls.status == LayerStatus::Failed {
        return Ok(build_skipped_after_tls(
            profile_name,
            url,
            transport,
            dns,
            tcp,
            tls,
            "TLS negotiation failed",
        ));
    }

    let http = to_layer_report(runner.check_http(url, effective_proxy));
    let conclusion = if http.status == LayerStatus::Success {
        "HTTP request completed".to_owned()
    } else {
        http.summary.clone()
    };

    Ok(CheckReport {
        target_url: url.into(),
        profile_name: profile_name.into(),
        transport,
        dns,
        tcp,
        tls,
        http,
        conclusion,
    })
}

pub fn build_check_report(
    profile_name: &str,
    url: &str,
    settings: &ProxySettings,
    runner: &dyn Runner,
) -> Result<CheckReport, String> {
    let effective = select_effective_proxy(url, settings)?;
    check_url_with_runner(profile_name, url, effective.value.as_deref(), runner)
}

fn to_layer_report(outcome: RunnerOutcome) -> LayerReport {
    LayerReport {
        status: if outcome.success {
            LayerStatus::Success
        } else {
            LayerStatus::Failed
        },
        summary: outcome.summary,
        detail: outcome.detail,
    }
}

fn skipped(summary: impl Into<String>) -> LayerReport {
    LayerReport {
        status: LayerStatus::Skipped,
        summary: summary.into(),
        detail: String::new(),
    }
}

fn build_skipped_after_dns(
    profile_name: &str,
    url: &str,
    transport: EffectiveProxy,
    dns: LayerReport,
    conclusion: &str,
) -> CheckReport {
    CheckReport {
        target_url: url.into(),
        profile_name: profile_name.into(),
        transport,
        dns,
        tcp: skipped("Skipped after DNS failure"),
        tls: skipped("Skipped after DNS failure"),
        http: skipped("Skipped after DNS failure"),
        conclusion: conclusion.into(),
    }
}

fn build_skipped_after_tcp(
    profile_name: &str,
    url: &str,
    transport: EffectiveProxy,
    dns: LayerReport,
    tcp: LayerReport,
    conclusion: &str,
) -> CheckReport {
    CheckReport {
        target_url: url.into(),
        profile_name: profile_name.into(),
        transport,
        dns,
        tcp,
        tls: skipped("Skipped after TCP failure"),
        http: skipped("Skipped after TCP failure"),
        conclusion: conclusion.into(),
    }
}

fn build_skipped_after_tls(
    profile_name: &str,
    url: &str,
    transport: EffectiveProxy,
    dns: LayerReport,
    tcp: LayerReport,
    tls: LayerReport,
    conclusion: &str,
) -> CheckReport {
    CheckReport {
        target_url: url.into(),
        profile_name: profile_name.into(),
        transport,
        dns,
        tcp,
        tls,
        http: skipped("Skipped after TLS failure"),
        conclusion: conclusion.into(),
    }
}
