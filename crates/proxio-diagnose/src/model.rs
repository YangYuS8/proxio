#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerStatus {
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Direct,
    Proxied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveProxy {
    pub mode: TransportMode,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerReport {
    pub status: LayerStatus,
    pub summary: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    pub target_url: String,
    pub profile_name: String,
    pub transport: EffectiveProxy,
    pub dns: LayerReport,
    pub tcp: LayerReport,
    pub tls: LayerReport,
    pub http: LayerReport,
    pub conclusion: String,
}
