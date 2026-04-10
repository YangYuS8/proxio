use proxio_adapters::apply::{ApplyEnvironment, apply_plan, preview_plan};
use proxio_adapters::command_runner::{CommandRunner, CommandSpec, CommandStatus};
use proxio_core::config::{ProxioConfig, ProxySettings};

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

#[test]
fn previews_command_targets() {
    let dir = tempfile::tempdir().unwrap();
    let env = ApplyEnvironment::for_root(dir.path());
    let plan = ProxioConfig {
        proxy: ProxySettings {
            http_proxy: Some("http://127.0.0.1:7890".into()),
            https_proxy: None,
            all_proxy: None,
            no_proxy: vec![],
        },
    }
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

    let results = apply_plan(&plan, &env, Some(&runner)).unwrap();
    assert!(results.iter().any(|item| item.target_name == "git"));
    assert!(results.iter().any(|item| item.target_name == "npm"));
    assert!(results.iter().any(|item| item.target_name == "pnpm"));
}
