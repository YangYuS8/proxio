use proxio_core::config::{ProxioConfig, ProxySettings};
use proxio_core::plan::TargetKind;

#[test]
fn normalizes_no_proxy_and_builds_all_targets() {
    let settings = ProxySettings {
        http_proxy: Some("http://127.0.0.1:7890".into()),
        https_proxy: Some("http://127.0.0.1:7890".into()),
        all_proxy: Some("socks5://127.0.0.1:7891".into()),
        no_proxy: vec![" localhost ".into(), "127.0.0.1".into(), "localhost".into()],
    };
    let config =
        ProxioConfig::new_with_profiles(Some("default".into()), [("default".into(), settings)]);

    let plan = config
        .build_plan_for_current_profile()
        .expect("plan should build");
    assert_eq!(
        config
            .current_profile()
            .expect("profile should resolve")
            .1
            .normalized_no_proxy(),
        vec!["127.0.0.1", "localhost"]
    );
    assert_eq!(plan.operations.len(), 5);
    assert!(
        plan.operations
            .iter()
            .all(|op| op.entries.iter().all(|entry| entry.value.is_set()))
    );
    assert!(
        plan.operations
            .iter()
            .any(|op| op.target == TargetKind::ShellEnv)
    );
    assert!(
        plan.operations
            .iter()
            .any(|op| op.target == TargetKind::SystemdUserEnv)
    );
    assert!(
        plan.operations
            .iter()
            .any(|op| op.target == TargetKind::Git)
    );
    assert!(
        plan.operations
            .iter()
            .any(|op| op.target == TargetKind::Npm)
    );
    assert!(
        plan.operations
            .iter()
            .any(|op| op.target == TargetKind::Pnpm)
    );
}

#[test]
fn rejects_invalid_proxy_url() {
    let config = ProxioConfig::new_with_profiles(
        Some("default".into()),
        [(
            "default".into(),
            ProxySettings {
                http_proxy: Some("not-a-url".into()),
                https_proxy: None,
                all_proxy: None,
                no_proxy: vec![],
            },
        )],
    );

    let error = config
        .build_plan_for_current_profile()
        .expect_err("invalid url should fail");
    assert!(error.to_string().contains("http_proxy"));
}
