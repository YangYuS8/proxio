use proxio_core::ProxySettings;
use proxio_diagnose::{TransportMode, select_effective_proxy};

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
