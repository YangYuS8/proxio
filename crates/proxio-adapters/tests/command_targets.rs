use proxio_adapters::apply::{ApplyEnvironment, apply_plan, preview_plan};
use proxio_adapters::command_runner::{CommandRunner, CommandSpec, CommandStatus};
use proxio_core::config::{ProxioConfig, ProxySettings};

fn proxy_config(http_proxy: Option<&str>, https_proxy: Option<&str>) -> ProxioConfig {
    ProxioConfig::new_with_profiles(
        Some("default".into()),
        [(
            "default".into(),
            ProxySettings {
                http_proxy: http_proxy.map(str::to_owned),
                https_proxy: https_proxy.map(str::to_owned),
                all_proxy: None,
                no_proxy: vec![],
            },
        )],
    )
}

#[derive(Default)]
struct FakeRunner {
    commands: std::sync::Mutex<Vec<CommandSpec>>,
}

impl CommandRunner for FakeRunner {
    fn command_exists(&self, _program: &str) -> bool {
        true
    }

    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String> {
        self.commands.lock().unwrap().push(spec.clone());
        Ok(CommandStatus {
            success: true,
            stderr: String::new(),
        })
    }
}

struct MissingGitUnsetRunner;

impl CommandRunner for MissingGitUnsetRunner {
    fn command_exists(&self, _program: &str) -> bool {
        true
    }

    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String> {
        if spec.program == "git" && spec.args.iter().any(|arg| arg == "--unset") {
            return Ok(CommandStatus {
                success: false,
                stderr: "error: No such section or key".into(),
            });
        }

        Ok(CommandStatus {
            success: true,
            stderr: String::new(),
        })
    }
}

struct SilentMissingGitUnsetRunner;

impl CommandRunner for SilentMissingGitUnsetRunner {
    fn command_exists(&self, _program: &str) -> bool {
        true
    }

    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String> {
        if spec.program == "git" && spec.args.iter().any(|arg| arg == "--unset") {
            return Ok(CommandStatus {
                success: false,
                stderr: String::new(),
            });
        }

        Ok(CommandStatus {
            success: true,
            stderr: String::new(),
        })
    }
}

#[test]
fn previews_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxy_config(Some("http://127.0.0.1:7890"), None)
        .build_plan()
        .unwrap();

    let preview = preview_plan(&plan, &env, Some(&FakeRunner::default())).unwrap();
    assert!(preview.iter().any(|item| item.target_name == "git"));
    assert!(preview.iter().any(|item| item.target_name == "npm"));
    assert!(preview.iter().any(|item| item.target_name == "pnpm"));
}

#[test]
fn applies_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let runner = FakeRunner::default();
    let plan = proxy_config(Some("http://127.0.0.1:7890"), Some("http://127.0.0.1:7890"))
        .build_plan()
        .unwrap();

    let results = apply_plan(&plan, &env, Some(&runner)).unwrap();
    assert!(results.iter().any(|item| item.target_name == "git"));
    assert!(results.iter().any(|item| item.target_name == "npm"));
    assert!(results.iter().any(|item| item.target_name == "pnpm"));
}

#[test]
fn applies_disable_plan_to_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let runner = FakeRunner::default();
    let plan = proxio_core::ProxioConfig::build_disable_plan();

    let results = apply_plan(&plan, &env, Some(&runner)).unwrap();
    assert!(
        results
            .iter()
            .any(|item| item.target_name == "git" && item.success)
    );

    let commands = runner.commands.lock().unwrap();
    assert!(
        commands
            .iter()
            .any(|spec| spec.args.iter().any(|arg| arg == "--unset"))
    );
    assert!(
        commands
            .iter()
            .any(|spec| spec.args.iter().any(|arg| arg == "delete"))
    );
}

#[test]
fn ignores_missing_git_keys_during_disable() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxio_core::ProxioConfig::build_disable_plan();

    let results = apply_plan(&plan, &env, Some(&MissingGitUnsetRunner)).unwrap();
    assert!(
        results
            .iter()
            .any(|item| item.target_name == "git" && item.success)
    );
}

#[test]
fn ignores_silent_missing_git_keys_during_disable() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = proxio_core::ProxioConfig::build_disable_plan();

    let results = apply_plan(&plan, &env, Some(&SilentMissingGitUnsetRunner)).unwrap();
    assert!(
        results
            .iter()
            .any(|item| item.target_name == "git" && item.success)
    );
}
