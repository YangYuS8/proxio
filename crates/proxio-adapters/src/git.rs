use crate::command_runner::CommandSpec;
use proxio_core::{PlannedEntryValue, PlannedOperation};

pub fn specs(operation: &PlannedOperation) -> Vec<CommandSpec> {
    operation
        .entries
        .iter()
        .filter_map(|entry| match entry.key.as_str() {
            "http_proxy" => Some(spec("http.proxy", &entry.value)),
            "https_proxy" => Some(spec("https.proxy", &entry.value)),
            _ => None,
        })
        .collect()
}

fn spec(key: &str, value: &PlannedEntryValue) -> CommandSpec {
    let args = match value {
        PlannedEntryValue::Set(value) => vec![
            "config".into(),
            "--global".into(),
            key.into(),
            value.clone(),
        ],
        PlannedEntryValue::Unset => vec![
            "config".into(),
            "--global".into(),
            "--unset".into(),
            key.into(),
        ],
    };

    CommandSpec {
        program: "git".into(),
        args,
    }
}
