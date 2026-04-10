use proxio_core::PlannedOperation;

fn systemd_escape(value: &str) -> String {
    value.replace('"', "\\\"")
}

pub fn render(operation: &PlannedOperation) -> String {
    operation
        .entries
        .iter()
        .map(|(key, value)| format!("{}=\"{}\"", key, systemd_escape(value)))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
