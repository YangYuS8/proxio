use std::fs;
use std::path::{Path, PathBuf};

use proxio_adapters::{ApplyEnvironment, ApplyResultItem};
use proxio_core::{ProxioConfig, ProxySettings};
use proxio_diagnose::CheckReport;

#[derive(Debug, Clone)]
pub struct LoadedState {
    pub profile_names: Vec<String>,
    pub current_profile: Option<String>,
    pub mode_summary: String,
}

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub message: String,
}

pub trait UiServices {
    fn load(&self) -> Result<LoadedState, String>;
    fn use_profile(&self, name: &str) -> Result<LoadedState, String>;
    fn apply(&self) -> Result<ActionResult, String>;
    fn disable(&self) -> Result<ActionResult, String>;
    fn check(&self, url: &str) -> Result<CheckReport, String>;
}

#[derive(Debug, Clone)]
pub struct RealServices {
    root: PathBuf,
}

impl RealServices {
    pub fn new() -> Result<Self, String> {
        let root = if let Ok(path) = std::env::var("PROXIO_HOME") {
            PathBuf::from(path)
        } else {
            PathBuf::from(std::env::var("HOME").map_err(|err| err.to_string())?)
        };
        Ok(Self { root })
    }
}

impl UiServices for RealServices {
    fn load(&self) -> Result<LoadedState, String> {
        let config = read_or_default(&config_path(&self.root))?;
        Ok(loaded_state_from_config(&config))
    }

    fn use_profile(&self, name: &str) -> Result<LoadedState, String> {
        let path = config_path(&self.root);
        let mut config = read_or_default(&path)?;
        config.profile(name).map_err(|err| err.to_string())?;
        config.current_profile = Some(name.to_owned());
        write_config(&path, &config)?;
        Ok(loaded_state_from_config(&config))
    }

    fn apply(&self) -> Result<ActionResult, String> {
        let path = config_path(&self.root);
        let config = read_or_default(&path)?;
        let plan = config
            .build_plan_for_current_profile()
            .map_err(|err| err.to_string())?;
        let env = ApplyEnvironment::for_root(&self.root);
        let items = proxio_adapters::apply_plan(&plan, &env, None)?;
        Ok(action_result_from_items("applied profile", items))
    }

    fn disable(&self) -> Result<ActionResult, String> {
        let plan = ProxioConfig::build_disable_plan();
        let env = ApplyEnvironment::for_root(&self.root);
        let items = proxio_adapters::apply_plan(&plan, &env, None)?;
        Ok(action_result_from_items("disabled managed settings", items))
    }

    fn check(&self, url: &str) -> Result<CheckReport, String> {
        let config = read_or_default(&config_path(&self.root))?;
        let (profile_name, settings) = config.current_profile().map_err(|err| err.to_string())?;
        proxio_diagnose::build_check_report(
            profile_name,
            url,
            settings,
            &proxio_diagnose::RealRunner,
        )
    }
}

fn config_path(root: &Path) -> PathBuf {
    proxio_adapters::paths::proxio_config_dir(root).join("config.toml")
}

fn read_config(path: &Path) -> Result<ProxioConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {}", path.display(), err))?;
    toml::from_str(&content).map_err(|err| format!("failed to parse {}: {}", path.display(), err))
}

fn read_or_default(path: &Path) -> Result<ProxioConfig, String> {
    if path.exists() {
        read_config(path)
    } else {
        Ok(ProxioConfig::new_with_profiles(None, std::iter::empty()))
    }
}

fn write_config(path: &Path, config: &ProxioConfig) -> Result<(), String> {
    let content = toml::to_string_pretty(config).map_err(|err| err.to_string())?;
    proxio_adapters::file_ops::atomic_write(path, &content)
}

fn loaded_state_from_config(config: &ProxioConfig) -> LoadedState {
    let current_profile = config.current_profile.clone();

    LoadedState {
        profile_names: config
            .profile_names()
            .into_iter()
            .map(str::to_owned)
            .collect(),
        current_profile,
        mode_summary: mode_summary(config),
    }
}

fn mode_summary(config: &ProxioConfig) -> String {
    match config.current_profile() {
        Ok((_, settings)) => summarize_settings(settings),
        Err(_) => "no profile selected".to_owned(),
    }
}

fn summarize_settings(settings: &ProxySettings) -> String {
    if settings.http_proxy.is_some()
        || settings.https_proxy.is_some()
        || settings.all_proxy.is_some()
    {
        "proxied".to_owned()
    } else {
        "direct".to_owned()
    }
}

fn action_result_from_items(message: &str, items: Vec<ApplyResultItem>) -> ActionResult {
    let success_count = items.iter().filter(|item| item.success).count();
    let skipped_count = items.iter().filter(|item| item.skipped).count();
    let failed_count = items
        .iter()
        .filter(|item| !item.success && !item.skipped)
        .count();

    ActionResult {
        success_count,
        skipped_count,
        failed_count,
        message: message.to_owned(),
    }
}
