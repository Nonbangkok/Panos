use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub extensions: Vec<String>,
    pub patterns: Vec<String>,
    pub destination: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub source_dir: PathBuf,
    pub rules: Vec<Rule>,
    pub watch_mode: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(help_template = "\
{before-help}
name: {name}
description: {about}
version: {version}
author: {author-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}
")]
#[command(arg_required_else_help = true)]
struct Args {
    /// Path to the configuration file (panos.toml)
    #[arg(short, long, default_value = "panos.toml")]
    config: PathBuf,

    /// Override the source directory to organize
    #[arg(short, long)]
    source: Option<PathBuf>,

    /// Run without moving files (only show what would happen)
    #[arg(short, long)]
    dry_run: bool,

    /// Run in watch mode (background daemon)
    #[arg(short, long)]
    watch: bool,
}

fn main() -> Result<()> {
    // Initialize logging
    let subscriber: FmtSubscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args: Args = Args::parse();

    info!("Starting PANOS...");
    info!("Config file: {:?}", args.config);
    
    // Load config
    let mut config: Config = load_config(&args.config)?;
    
    // CLI override for source directory
    if let Some(source) = args.source {
        config.source_dir = source;
    }

    info!("Source directory: {:?}", config.source_dir);
    
    if args.dry_run {
        info!("Dry run mode enabled. Files will not be moved.");
    }

    organize(&config, args.dry_run)?;

    Ok(())
}

fn organize(config: &Config, dry_run: bool) -> Result<()> {
    use walkdir::WalkDir;

    if !config.source_dir.exists() {
        return Err(anyhow::anyhow!("Source directory does not exist: {:?}", config.source_dir));
    }

    info!("Scanning {:?}...", config.source_dir);

    for entry in WalkDir::new(&config.source_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e: std::result::Result<walkdir::DirEntry, walkdir::Error>| e.ok())
    {
        if entry.file_type().is_file() {
            let path: &std::path::Path = entry.path();
            if let Some(rule) = find_rule_for_file(path, &config.rules) {
                move_file(path, &rule.destination, dry_run)?;
            }
        }
    }

    Ok(())
}

fn find_rule_for_file<'a>(path: &std::path::Path, rules: &'a [Rule]) -> Option<&'a Rule> {
    let extension: String = path.extension()?.to_str()?.to_lowercase();
    
    for rule in rules {
        if rule.extensions.iter().any(|ext: &String| ext.to_lowercase() == extension) {
            return Some(rule);
        }
    }
    None
}

fn move_file(source: &std::path::Path, dest_dir: &std::path::Path, dry_run: bool) -> Result<()> {
    let file_name: &std::ffi::OsStr = source.file_name().ok_or_else(|| anyhow::anyhow!("Could not get file name"))?;
    let mut dest_path: PathBuf = dest_dir.join(file_name);

    if !dry_run {
        std::fs::create_dir_all(dest_dir)?;
    }

    // Handle conflict
    if dest_path.exists() {
        let stem: &str = source.file_stem().and_then(|s: &std::ffi::OsStr| s.to_str()).unwrap_or("file");
        let extension: &str = source.extension().and_then(|e: &std::ffi::OsStr| e.to_str()).unwrap_or("");
        
        let mut count: i32 = 1;
        while dest_path.exists() {
            let new_name: String = if extension.is_empty() {
                format!("{}_{}", stem, count)
            } else {
                format!("{}_{}.{}", stem, count, extension)
            };
            dest_path = dest_dir.join(new_name);
            count += 1;
        }
    }

    info!("Moving {:?} to {:?}", source, dest_path);

    if !dry_run {
        std::fs::rename(source, dest_path)?;
    }

    Ok(())
}

fn load_config(path: &std::path::Path) -> Result<Config> {
    let content: String = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
