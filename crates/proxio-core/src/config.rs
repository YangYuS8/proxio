use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::plan::ApplyPlan;
use crate::validate::ValidationError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProxioConfig {
    #[serde(default)]
    pub current_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProxySettings>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ConfigRepr {
    Current(CurrentConfig),
    Legacy(LegacyConfig),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrentConfig {
    #[serde(default)]
    current_profile: Option<String>,
    #[serde(default)]
    profiles: BTreeMap<String, ProxySettings>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyConfig {
    proxy: ProxySettings,
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
    pub fn new_with_profiles(
        current_profile: Option<String>,
        profiles: impl IntoIterator<Item = (String, ProxySettings)>,
    ) -> Self {
        Self {
            current_profile,
            profiles: profiles.into_iter().collect(),
        }
    }

    pub fn current_profile(&self) -> Result<(&str, &ProxySettings), ValidationError> {
        let name = self
            .current_profile
            .as_deref()
            .ok_or_else(|| ValidationError {
                field: "current_profile",
                message: "no current profile selected".into(),
            })?;
        let settings = self.profile(name)?;
        Ok((name, settings))
    }

    pub fn profile_names(&self) -> Vec<&str> {
        self.profiles.keys().map(String::as_str).collect()
    }

    pub fn profile(&self, name: &str) -> Result<&ProxySettings, ValidationError> {
        validate_profile_name(name)?;
        self.profiles.get(name).ok_or_else(|| ValidationError {
            field: "current_profile",
            message: format!("unknown profile: {name}"),
        })
    }

    pub fn build_plan(&self) -> Result<ApplyPlan, ValidationError> {
        self.build_plan_for_current_profile()
    }

    pub fn build_plan_for_current_profile(&self) -> Result<ApplyPlan, ValidationError> {
        let (name, _) = self.current_profile()?;
        self.build_plan_for_profile(name)
    }

    pub fn build_plan_for_profile(&self, name: &str) -> Result<ApplyPlan, ValidationError> {
        let profile = self.profile(name)?;
        crate::validate::validate_proxy_settings(profile)?;
        let plan = ApplyPlan::from_settings(profile);
        if plan.is_empty() {
            Ok(ApplyPlan::disable())
        } else {
            Ok(plan)
        }
    }

    pub fn build_disable_plan() -> ApplyPlan {
        ApplyPlan::disable()
    }
}

fn validate_profile_name(name: &str) -> Result<(), ValidationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ValidationError {
            field: "profile",
            message: "profile name cannot be empty".into(),
        });
    }

    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        Ok(())
    } else {
        Err(ValidationError {
            field: "profile",
            message: format!("invalid profile name: {name}"),
        })
    }
}

impl<'de> Deserialize<'de> for ProxioConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match ConfigRepr::deserialize(deserializer)? {
            ConfigRepr::Current(config) => Ok(Self {
                current_profile: config.current_profile,
                profiles: config.profiles,
            }),
            ConfigRepr::Legacy(LegacyConfig { proxy }) => {
                let mut profiles = BTreeMap::new();
                profiles.insert("default".into(), proxy);
                Ok(Self {
                    current_profile: Some("default".into()),
                    profiles,
                })
            }
        }
    }
}
