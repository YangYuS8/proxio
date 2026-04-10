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
pub struct PlannedOperation {
    pub target: TargetKind,
    pub entries: Vec<(String, String)>,
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
            entries.push(("http_proxy".into(), value.to_owned()));
        }
        if let Some(value) = settings
            .https_proxy
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            entries.push(("https_proxy".into(), value.to_owned()));
        }
        if let Some(value) = settings
            .all_proxy
            .as_ref()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
        {
            entries.push(("all_proxy".into(), value.to_owned()));
        }

        let no_proxy = settings.normalized_no_proxy().join(",");
        if !no_proxy.is_empty() {
            entries.push(("no_proxy".into(), no_proxy));
        }

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
