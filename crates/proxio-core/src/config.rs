use serde::{Deserialize, Serialize};

use crate::plan::ApplyPlan;
use crate::validate::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxioConfig {
    pub proxy: ProxySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxySettings {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub all_proxy: Option<String>,
    #[serde(default)]
    pub no_proxy: Vec<String>,
}

impl ProxySettings {
    pub fn normalized_no_proxy(&self) -> Vec<String> {
        let mut values: Vec<String> = self
            .no_proxy
            .iter()
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        values.sort();
        values.dedup();
        values
    }
}

impl ProxioConfig {
    pub fn build_plan(&self) -> Result<ApplyPlan, ValidationError> {
        crate::validate::validate_proxy_settings(&self.proxy)?;
        Ok(ApplyPlan::from_settings(&self.proxy))
    }
}
