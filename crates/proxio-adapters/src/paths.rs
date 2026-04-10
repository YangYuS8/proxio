use std::path::{Path, PathBuf};

pub fn proxio_config_dir(root: &Path) -> PathBuf {
    root.join(".config/proxio")
}

pub fn proxio_shell_env_path(root: &Path) -> PathBuf {
    proxio_config_dir(root).join("env/proxy.env")
}

pub fn systemd_user_env_path(root: &Path) -> PathBuf {
    root.join(".config/environment.d/proxio-proxy.conf")
}
