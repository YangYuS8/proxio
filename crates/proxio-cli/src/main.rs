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
            let config = ProxioConfig {
                proxy: ProxySettings {
                    http_proxy,
                    https_proxy,
                    all_proxy,
                    no_proxy,
                },
            };
            write_config(&path, &config)?;
            println!("saved {}", path.display());
        }
        Commands::Show => {
            let config = read_config(&path)?;
            print_config(&config)?;
        }
        Commands::Preview => {
            let config = read_config(&path)?;
            let plan = config.build_plan().map_err(|err| err.to_string())?;
            let env = proxio_adapters::ApplyEnvironment::for_root(&root);
            for item in proxio_adapters::preview_plan(&plan, &env, None)? {
                println!("[{}] {}", item.target_name, item.summary);
            }
        }
        Commands::Apply => {
            let config = read_config(&path)?;
            let plan = config.build_plan().map_err(|err| err.to_string())?;
            let env = proxio_adapters::ApplyEnvironment::for_root(&root);
            for item in proxio_adapters::apply_plan(&plan, &env, None)? {
                println!("[{}] {}", item.target_name, item.message);
            }
        }
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
