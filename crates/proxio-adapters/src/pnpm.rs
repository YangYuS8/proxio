use crate::command_runner::CommandSpec;
use proxio_core::PlannedOperation;

pub fn specs(operation: &PlannedOperation) -> Vec<CommandSpec> {
    operation
        .entries
        .iter()
        .filter_map(|(key, value)| match key.as_str() {
            "http_proxy" => Some(CommandSpec {
                program: "pnpm".into(),
                args: vec!["config".into(), "set".into(), "proxy".into(), value.clone()],
            }),
            "https_proxy" => Some(CommandSpec {
                program: "pnpm".into(),
                args: vec![
                    "config".into(),
                    "set".into(),
                    "https-proxy".into(),
                    value.clone(),
                ],
            }),
            _ => None,
        })
        .collect()
}
