use proxio_adapters::apply::{ApplyEnvironment, apply_plan, preview_plan};
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
    assert!(
        preview
            .iter()
            .any(|item| item.target_name == "systemd_user_env")
    );
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
    assert!(
        results
            .iter()
            .any(|item| item.target_name == "shell_env" && item.success)
    );
    assert!(
        results
            .iter()
            .any(|item| item.target_name == "systemd_user_env" && item.success)
    );
}
