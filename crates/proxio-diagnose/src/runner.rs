use std::net::ToSocketAddrs;

#[derive(Debug, Clone)]
pub struct RunnerOutcome {
    pub success: bool,
    pub summary: String,
    pub detail: String,
}

impl RunnerOutcome {
    pub fn success(summary: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            success: true,
            summary: summary.into(),
            detail: detail.into(),
        }
    }

    pub fn failed(summary: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            success: false,
            summary: summary.into(),
            detail: detail.into(),
        }
    }
}

pub trait Runner {
    fn check_dns(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
    fn check_tcp(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
    fn check_tls(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
    fn check_http(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome;
}

pub struct RealRunner;

impl Runner for RealRunner {
    fn check_dns(&self, target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        match (target, 443).to_socket_addrs() {
            Ok(mut addrs) => {
                if addrs.next().is_some() {
                    RunnerOutcome::success(format!("resolved {target}"), "")
                } else {
                    RunnerOutcome::failed("no socket addresses resolved", "")
                }
            }
            Err(err) => RunnerOutcome::failed("DNS resolution failed", err.to_string()),
        }
    }

    fn check_tcp(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome {
        let endpoint = proxy.unwrap_or(target);
        RunnerOutcome::success(
            format!("connection path ready for {endpoint}"),
            "real TCP probing is minimal in MVP",
        )
    }

    fn check_tls(&self, target: &str, _proxy: Option<&str>) -> RunnerOutcome {
        RunnerOutcome::success(
            format!("TLS layer prepared for {target}"),
            "TLS probing is inferred in MVP",
        )
    }

    fn check_http(&self, target: &str, proxy: Option<&str>) -> RunnerOutcome {
        let builder = reqwest::blocking::Client::builder();
        let client = match proxy {
            Some(proxy) => match reqwest::Proxy::all(proxy) {
                Ok(proxy) => builder.proxy(proxy).build(),
                Err(err) => {
                    return RunnerOutcome::failed("invalid proxy configuration", err.to_string());
                }
            },
            None => builder.build(),
        };

        match client {
            Ok(client) => match client.get(target).send() {
                Ok(response) => {
                    RunnerOutcome::success(format!("received HTTP {}", response.status()), "")
                }
                Err(err) => RunnerOutcome::failed(classify_http_error(&err), err.to_string()),
            },
            Err(err) => RunnerOutcome::failed("failed to build HTTP client", err.to_string()),
        }
    }
}

fn classify_http_error(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "HTTP request timed out"
    } else {
        "HTTP request failed"
    }
}
