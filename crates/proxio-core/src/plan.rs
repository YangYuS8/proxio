use crate::config::ProxySettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    ShellEnv,
    SystemdUserEnv,
    Git,
    Npm,
    Pnpm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedEntryValue {
    Set(String),
    Unset,
}

impl PlannedEntryValue {
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }

    pub fn is_unset(&self) -> bool {
        matches!(self, Self::Unset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedEntry {
    pub key: String,
    pub value: PlannedEntryValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedOperation {
    pub target: TargetKind,
    pub entries: Vec<PlannedEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyPlan {
    pub operations: Vec<PlannedOperation>,
}

impl ApplyPlan {
    pub fn from_settings(settings: &ProxySettings) -> Self {
        let mut entries = Vec::new();

        if let Some(value) = settings
            .http_proxy
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            entries.push(PlannedEntry {
                key: "http_proxy".into(),
                value: PlannedEntryValue::Set(value.to_owned()),
            });
        }
        if let Some(value) = settings
            .https_proxy
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            entries.push(PlannedEntry {
                key: "https_proxy".into(),
                value: PlannedEntryValue::Set(value.to_owned()),
            });
        }
        if let Some(value) = settings
            .all_proxy
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            entries.push(PlannedEntry {
                key: "all_proxy".into(),
                value: PlannedEntryValue::Set(value.to_owned()),
            });
        }

        let no_proxy = settings.normalized_no_proxy().join(",");
        if !no_proxy.is_empty() {
            entries.push(PlannedEntry {
                key: "no_proxy".into(),
                value: PlannedEntryValue::Set(no_proxy),
            });
        }

        if entries.is_empty() {
            return Self::disable();
        }

        Self::with_entries(entries)
    }

    pub fn disable() -> Self {
        let entries = ["http_proxy", "https_proxy", "all_proxy", "no_proxy"]
            .into_iter()
            .map(|key| PlannedEntry {
                key: key.into(),
                value: PlannedEntryValue::Unset,
            })
            .collect();

        Self::with_entries(entries)
    }

    pub fn is_empty(&self) -> bool {
        self.operations
            .iter()
            .all(|operation| operation.entries.is_empty())
    }
}

impl ApplyPlan {
    fn with_entries(entries: Vec<PlannedEntry>) -> Self {
        let operations = [
            TargetKind::ShellEnv,
            TargetKind::SystemdUserEnv,
            TargetKind::Git,
            TargetKind::Npm,
            TargetKind::Pnpm,
        ]
        .into_iter()
        .map(|target| PlannedOperation {
            target,
            entries: entries.clone(),
        })
        .collect();

        Self { operations }
    }
}
