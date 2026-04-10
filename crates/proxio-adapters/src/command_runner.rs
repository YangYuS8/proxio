#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommandStatus {
    pub success: bool,
    pub stderr: String,
}

pub trait CommandRunner {
    fn command_exists(&self, program: &str) -> bool;
    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String>;
}

pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn command_exists(&self, program: &str) -> bool {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {} >/dev/null 2>&1", program))
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn run(&self, spec: &CommandSpec) -> Result<CommandStatus, String> {
        let output = std::process::Command::new(&spec.program)
            .args(&spec.args)
            .output()
            .map_err(|err| err.to_string())?;

        Ok(CommandStatus {
            success: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}
