use proxio_adapters::apply::{apply_plan, preview_plan, ApplyEnvironment};
use proxio_adapters::command_runner::{CommandRunner, CommandSpec, CommandStatus};
use proxio_core::config::{ProxioConfig, ProxySettings};

fn proxy_config(
    http_proxy: Option<&str>,
    https_proxy: Option<&str>,
    no_proxy: &[&str],
) -> ProxioConfig {
    ProxioConfig::new_with_profiles(
        Some("default".into()),
        [(
            "default".into(),
            ProxySettings {
                http_proxy: http_proxy.map(str::to_owned),
                https_proxy: https_proxy.map(str::to_owned),
                all_proxy: None,
                no_proxy: no_proxy.iter().map(|value| (*value).to_owned()).collect(),
            },
        )],
    )
}

struct FakeRunner;

impl CommandRunner for FakeRunner {
    fn command_exists(&self, _program: &str) -> bool {
        true
    }

    fn run(&self, _spec: &CommandSpec) -> Result<CommandStatus, String> {
        Ok(CommandStatus {
            success: true,
            stderr: String::new(),
        })
    }
}

#[test]
fn previews_shell_and_systemd_files() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxy_config(Some("http://127.0.0.1:7890"), None, &["localhost"])
        .build_plan()
        .unwrap();

    let preview = preview_plan(&plan, &env, None).unwrap();
    assert!(preview.iter().any(|item| item.target_name == "shell_env"));
    assert!(preview
        .iter()
        .any(|item| item.target_name == "systemd_user_env"));
}

#[test]
fn applies_shell_and_systemd_files() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxy_config(
        Some("http://127.0.0.1:7890"),
        Some("http://127.0.0.1:7890"),
        &[],
    )
    .build_plan()
    .unwrap();

    let results = apply_plan(&plan, &env, Some(&FakeRunner)).unwrap();
    assert!(results
        .iter()
        .any(|item| item.target_name == "shell_env" && item.success));
    assert!(results
        .iter()
        .any(|item| item.target_name == "systemd_user_env" && item.success));
}

#[test]
fn applies_disable_plan_as_empty_managed_files() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxio_core::ProxioConfig::build_disable_plan();

    let results = apply_plan(&plan, &env, None).unwrap();
    assert!(results.iter().all(|item| item.success || item.skipped));

    let shell_path = proxio_adapters::paths::proxio_shell_env_path(dir.path());
    let systemd_path = proxio_adapters::paths::systemd_user_env_path(dir.path());
    assert_eq!(std::fs::read_to_string(shell_path).unwrap(), "");
    assert_eq!(std::fs::read_to_string(systemd_path).unwrap(), "");
}
