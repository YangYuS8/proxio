use proxio_core::{PlannedEntryValue, PlannedOperation};

fn systemd_escape(value: &str) -> String {
    value.replace('"', "\\\"")
}

pub fn render(operation: &PlannedOperation) -> String {
    let rendered = operation
        .entries
        .iter()
        .filter_map(|entry| match &entry.value {
            PlannedEntryValue::Set(value) => {
                Some(format!("{}=\"{}\"", entry.key, systemd_escape(value)))
            }
            PlannedEntryValue::Unset => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    if rendered.is_empty() {
        rendered
    } else {
        rendered + "\n"
    }
}
