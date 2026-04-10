use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use proxio_core::{ProxioConfig, ProxySettings};

#[derive(Parser)]
#[command(name = "proxio")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set {
        #[arg(long)]
        http_proxy: Option<String>,
        #[arg(long)]
        https_proxy: Option<String>,
        #[arg(long)]
        all_proxy: Option<String>,
        #[arg(long, value_delimiter = ',')]
        no_proxy: Vec<String>,
    },
    Show,
    Preview,
    Apply,
    Disable,
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    List,
    Add {
        name: String,
        #[arg(long)]
        http_proxy: Option<String>,
        #[arg(long)]
        https_proxy: Option<String>,
        #[arg(long)]
        all_proxy: Option<String>,
        #[arg(long, value_delimiter = ',')]
        no_proxy: Vec<String>,
    },
    Remove {
        name: String,
    },
    Use {
        name: String,
    },
    Current,
}

fn empty_config() -> ProxioConfig {
    ProxioConfig::new_with_profiles(None, std::iter::empty())
}

fn read_or_default(path: &Path) -> Result<ProxioConfig, String> {
    if path.exists() {
        read_config(path)
    } else {
        Ok(empty_config())
    }
}

fn proxy_settings(
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    all_proxy: Option<String>,
    no_proxy: Vec<String>,
) -> ProxySettings {
    ProxySettings {
        http_proxy,
        https_proxy,
        all_proxy,
        no_proxy,
    }
}

fn validate_profile_input(name: &str, settings: &ProxySettings) -> Result<(), String> {
    let config = ProxioConfig::new_with_profiles(
        Some(name.to_owned()),
        [(name.to_owned(), settings.clone())],
    );
    config
        .build_plan_for_current_profile()
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let root = proxio_root();
    let path = config_path(&root);

    match cli.command {
        Commands::Set {
            http_proxy,
            https_proxy,
            all_proxy,
            no_proxy,
        } => {
            let mut config = read_or_default(&path)?;
            let name = config
                .current_profile
                .clone()
                .unwrap_or_else(|| "default".into());
            config.profiles.insert(
                name.clone(),
                proxy_settings(http_proxy, https_proxy, all_proxy, no_proxy),
            );
            config.current_profile = Some(name);
            write_config(&path, &config)?;
            println!("saved {}", path.display());
        }
        Commands::Show => {
            let config = read_or_default(&path)?;
            print_config(&config)?;
        }
        Commands::Preview => {
            let config = read_or_default(&path)?;
            let plan = config
                .build_plan_for_current_profile()
                .map_err(|err| err.to_string())?;
            let env = proxio_adapters::ApplyEnvironment::for_root(&root);
            for item in proxio_adapters::preview_plan(&plan, &env, None)? {
                println!("[{}] {}", item.target_name, item.summary);
            }
        }
        Commands::Apply => {
            let config = read_or_default(&path)?;
            let plan = config
                .build_plan_for_current_profile()
                .map_err(|err| err.to_string())?;
            let env = proxio_adapters::ApplyEnvironment::for_root(&root);
            for item in proxio_adapters::apply_plan(&plan, &env, None)? {
                println!("[{}] {}", item.target_name, item.message);
            }
        }
        Commands::Disable => {
            let plan = proxio_core::ProxioConfig::build_disable_plan();
            let env = proxio_adapters::ApplyEnvironment::for_root(&root);
            for item in proxio_adapters::apply_plan(&plan, &env, None)? {
                println!("[{}] {}", item.target_name, item.message);
            }
        }
        Commands::Profile { command } => match command {
            ProfileCommands::List => {
                let config = read_or_default(&path)?;
                if config.profiles.is_empty() {
                    println!("no profiles configured");
                } else {
                    for name in config.profile_names() {
                        let marker = if config.current_profile.as_deref() == Some(name) {
                            "*"
                        } else {
                            " "
                        };
                        println!("{} {}", marker, name);
                    }
                }
            }
            ProfileCommands::Add {
                name,
                http_proxy,
                https_proxy,
                all_proxy,
                no_proxy,
            } => {
                let mut config = read_or_default(&path)?;
                if config.profiles.contains_key(&name) {
                    return Err(format!("profile already exists: {name}"));
                }

                let settings = proxy_settings(http_proxy, https_proxy, all_proxy, no_proxy);
                validate_profile_input(&name, &settings)?;
                config.profiles.insert(name.clone(), settings);
                if config.current_profile.is_none() {
                    config.current_profile = Some(name.clone());
                }
                write_config(&path, &config)?;
                println!("saved profile {}", name);
            }
            ProfileCommands::Remove { name } => {
                let mut config = read_or_default(&path)?;
                if config.current_profile.as_deref() == Some(name.as_str()) {
                    return Err(format!("cannot remove active profile: {name}"));
                }
                if config.profiles.remove(&name).is_none() {
                    return Err(format!("unknown profile: {name}"));
                }
                write_config(&path, &config)?;
                println!("removed profile {}", name);
            }
            ProfileCommands::Use { name } => {
                let mut config = read_or_default(&path)?;
                config.profile(&name).map_err(|err| err.to_string())?;
                config.current_profile = Some(name.clone());
                write_config(&path, &config)?;
                println!("using profile {}", name);
            }
            ProfileCommands::Current => {
                let config = read_or_default(&path)?;
                let (name, profile) = config.current_profile().map_err(|err| err.to_string())?;
                println!("current profile: {}", name);
                print!(
                    "{}",
                    toml::to_string_pretty(profile).map_err(|err| err.to_string())?
                );
            }
        },
    }

    Ok(())
}

fn proxio_root() -> PathBuf {
    if let Ok(path) = std::env::var("PROXIO_HOME") {
        return PathBuf::from(path);
    }

    let home = std::env::var("HOME").expect("HOME must be set");
    PathBuf::from(home)
}

fn config_path(root: &Path) -> PathBuf {
    proxio_adapters::paths::proxio_config_dir(root).join("config.toml")
}

fn read_config(path: &Path) -> Result<ProxioConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {}", path.display(), err))?;
    toml::from_str(&content).map_err(|err| format!("failed to parse {}: {}", path.display(), err))
}

fn write_config(path: &Path, config: &ProxioConfig) -> Result<(), String> {
    let content = toml::to_string_pretty(config).map_err(|err| err.to_string())?;
    proxio_adapters::file_ops::atomic_write(path, &content)
}

fn print_config(config: &ProxioConfig) -> Result<(), String> {
    let content = toml::to_string_pretty(config).map_err(|err| err.to_string())?;
    print!("{}", content);
    Ok(())
}
