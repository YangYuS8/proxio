use proxio_core::{PlannedEntryValue, PlannedOperation};

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn render(operation: &PlannedOperation) -> String {
    let rendered = operation
        .entries
        .iter()
        .filter_map(|entry| match &entry.value {
            PlannedEntryValue::Set(value) => {
                Some(format!("export {}={}", entry.key, shell_quote(value)))
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
