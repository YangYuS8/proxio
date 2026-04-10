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

    let (name, profile) = config
        .current_profile()
        .expect("current profile should resolve");
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
    let (name, profile) = config
        .current_profile()
        .expect("legacy current profile should resolve");
    assert_eq!(name, "default");
    assert_eq!(
        profile.https_proxy.as_deref(),
        Some("http://127.0.0.1:7890")
    );
}

#[test]
fn builds_disable_plan_with_unset_values() {
    let plan = ProxioConfig::build_disable_plan();
    assert_eq!(plan.operations.len(), 5);
    assert!(
        plan.operations
            .iter()
            .all(|op| op.entries.iter().all(|entry| entry.value.is_unset()))
    );
    assert!(plan.operations.iter().all(|op| {
        op.entries.iter().map(|entry| entry.key.as_str()).eq([
            "http_proxy",
            "https_proxy",
            "all_proxy",
            "no_proxy",
        ])
    }));
}

#[test]
fn empty_current_profile_builds_disable_plan() {
    let config = ProxioConfig::new_with_profiles(
        Some("direct".into()),
        [(
            "direct".into(),
            ProxySettings {
                http_proxy: None,
                https_proxy: None,
                all_proxy: None,
                no_proxy: vec![],
            },
        )],
    );

    let plan = config
        .build_plan_for_current_profile()
        .expect("empty profile should build disable plan");

    assert_eq!(plan, ProxioConfig::build_disable_plan());
}

#[test]
fn lists_profiles_stably_and_builds_plan_for_named_profile() {
    let config = ProxioConfig::new_with_profiles(
        Some("beta".into()),
        [
            (
                "z-last".into(),
                ProxySettings {
                    http_proxy: None,
                    https_proxy: None,
                    all_proxy: None,
                    no_proxy: vec![],
                },
            ),
            (
                "alpha".into(),
                ProxySettings {
                    http_proxy: Some("http://127.0.0.1:7000".into()),
                    https_proxy: None,
                    all_proxy: None,
                    no_proxy: vec![],
                },
            ),
            (
                "beta".into(),
                ProxySettings {
                    http_proxy: Some("http://127.0.0.1:7890".into()),
                    https_proxy: Some("http://127.0.0.1:7890".into()),
                    all_proxy: None,
                    no_proxy: vec!["localhost".into()],
                },
            ),
        ],
    );

    assert_eq!(config.profile_names(), vec!["alpha", "beta", "z-last"]);

    let profile = config.profile("alpha").expect("profile should resolve");
    assert_eq!(profile.http_proxy.as_deref(), Some("http://127.0.0.1:7000"));

    let named_plan = config
        .build_plan_for_profile("beta")
        .expect("named plan should build");
    let current_plan = config
        .build_plan_for_current_profile()
        .expect("current plan should build");
    assert_eq!(named_plan, current_plan);
}

#[test]
fn roundtrips_current_multi_profile_format() {
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
                    all_proxy: Some("socks5://127.0.0.1:7891".into()),
                    no_proxy: vec!["localhost".into(), "127.0.0.1".into()],
                },
            ),
        ],
    );

    let serialized = toml::to_string(&config).expect("config should serialize");
    let reparsed: ProxioConfig = toml::from_str(&serialized).expect("config should deserialize");

    assert_eq!(reparsed, config);
}

#[test]
fn rejects_mixed_current_and_legacy_config_shapes() {
    let content = r#"
current_profile = "proxy"

[profiles.proxy]
http_proxy = "http://127.0.0.1:7890"

[proxy]
https_proxy = "http://127.0.0.1:7890"
"#;

    toml::from_str::<ProxioConfig>(content).expect_err("mixed config shapes should be rejected");
}
