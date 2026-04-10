use std::path::{Path, PathBuf};

use proxio_core::{ApplyPlan, PlannedOperation, TargetKind};

use crate::command_runner::{CommandRunner, RealCommandRunner};

#[derive(Debug, Clone)]
pub struct ApplyEnvironment {
    pub root: PathBuf,
}

impl ApplyEnvironment {
    pub fn for_root(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreviewItem {
    pub target_name: &'static str,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct ApplyResultItem {
    pub target_name: &'static str,
    pub success: bool,
    pub skipped: bool,
    pub message: String,
}

pub fn preview_plan(
    plan: &ApplyPlan,
    env: &ApplyEnvironment,
    runner: Option<&dyn CommandRunner>,
) -> Result<Vec<PreviewItem>, String> {
    let default_runner = RealCommandRunner;
    let runner: &dyn CommandRunner = runner.unwrap_or(&default_runner);

    plan.operations
        .iter()
        .map(|operation| preview_operation(operation, env, runner))
        .collect()
}

pub fn apply_plan(
    plan: &ApplyPlan,
    env: &ApplyEnvironment,
    runner: Option<&dyn CommandRunner>,
) -> Result<Vec<ApplyResultItem>, String> {
    let default_runner = RealCommandRunner;
    let runner: &dyn CommandRunner = runner.unwrap_or(&default_runner);

    plan.operations
        .iter()
        .map(|operation| apply_operation(operation, env, runner))
        .collect()
}

fn preview_operation(
    operation: &PlannedOperation,
    env: &ApplyEnvironment,
    runner: &dyn CommandRunner,
) -> Result<PreviewItem, String> {
    match operation.target {
        TargetKind::ShellEnv => {
            let path = crate::paths::proxio_shell_env_path(&env.root);
            Ok(PreviewItem {
                target_name: "shell_env",
                summary: format!(
                    "write {}\n{}",
                    path.display(),
                    crate::shell_env::render(operation)
                ),
            })
        }
        TargetKind::SystemdUserEnv => {
            let path = crate::paths::systemd_user_env_path(&env.root);
            Ok(PreviewItem {
                target_name: "systemd_user_env",
                summary: format!(
                    "write {}\n{}",
                    path.display(),
                    crate::systemd_user_env::render(operation)
                ),
            })
        }
        TargetKind::Git => preview_commands("git", crate::git::specs(operation), runner),
        TargetKind::Npm => preview_optional_commands("npm", crate::npm::specs(operation), runner),
        TargetKind::Pnpm => {
            preview_optional_commands("pnpm", crate::pnpm::specs(operation), runner)
        }
    }
}

fn apply_operation(
    operation: &PlannedOperation,
    env: &ApplyEnvironment,
    runner: &dyn CommandRunner,
) -> Result<ApplyResultItem, String> {
    match operation.target {
        TargetKind::ShellEnv => {
            let path = crate::paths::proxio_shell_env_path(&env.root);
            crate::file_ops::atomic_write(&path, &crate::shell_env::render(operation))?;
            Ok(ApplyResultItem {
                target_name: "shell_env",
                success: true,
                skipped: false,
                message: format!("wrote {}", path.display()),
            })
        }
        TargetKind::SystemdUserEnv => {
            let path = crate::paths::systemd_user_env_path(&env.root);
            crate::file_ops::atomic_write(&path, &crate::systemd_user_env::render(operation))?;
            Ok(ApplyResultItem {
                target_name: "systemd_user_env",
                success: true,
                skipped: false,
                message: format!("wrote {}", path.display()),
            })
        }
        TargetKind::Git => run_commands("git", crate::git::specs(operation), runner),
        TargetKind::Npm => run_optional_commands("npm", crate::npm::specs(operation), runner),
        TargetKind::Pnpm => run_optional_commands("pnpm", crate::pnpm::specs(operation), runner),
    }
}

fn preview_commands(
    target_name: &'static str,
    specs: Vec<crate::command_runner::CommandSpec>,
    runner: &dyn CommandRunner,
) -> Result<PreviewItem, String> {
    if !runner.command_exists(target_name) {
        return Ok(PreviewItem {
            target_name,
            summary: format!("skipped: {} not found", target_name),
        });
    }

    Ok(PreviewItem {
        target_name,
        summary: specs
            .iter()
            .map(|spec| format!("{} {}", spec.program, spec.args.join(" ")))
            .collect::<Vec<_>>()
            .join("\n"),
    })
}

fn preview_optional_commands(
    target_name: &'static str,
    specs: Vec<crate::command_runner::CommandSpec>,
    runner: &dyn CommandRunner,
) -> Result<PreviewItem, String> {
    preview_commands(target_name, specs, runner)
}

fn run_commands(
    target_name: &'static str,
    specs: Vec<crate::command_runner::CommandSpec>,
    runner: &dyn CommandRunner,
) -> Result<ApplyResultItem, String> {
    if !runner.command_exists(target_name) {
        return Ok(ApplyResultItem {
            target_name,
            success: false,
            skipped: true,
            message: format!("skipped: {} not found", target_name),
        });
    }

    for spec in &specs {
        let status = runner.run(spec)?;
        if !status.success {
            return Ok(ApplyResultItem {
                target_name,
                success: false,
                skipped: false,
                message: format!("failed: {}", status.stderr.trim()),
            });
        }
    }

    Ok(ApplyResultItem {
        target_name,
        success: true,
        skipped: false,
        message: format!("applied {} command(s)", specs.len()),
    })
}

fn run_optional_commands(
    target_name: &'static str,
    specs: Vec<crate::command_runner::CommandSpec>,
    runner: &dyn CommandRunner,
) -> Result<ApplyResultItem, String> {
    run_commands(target_name, specs, runner)
}
