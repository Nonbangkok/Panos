//! Configuration loading and parsing

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Rule {
    pub name: String,
    pub extensions: Vec<String>,
    pub patterns: Vec<String>,
    pub semantic_label: Option<String>,
    pub destination: PathBuf,

    #[serde(skip)]
    pub glob_set: Option<globset::GlobSet>,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            name: String::new(),
            extensions: vec![],
            patterns: vec![],
            semantic_label: None,
            destination: PathBuf::new(),
            glob_set: None,
        }
    }
}

impl Rule {
    pub fn matches(&self, filename: &str, extension: &str) -> bool {
        // check pattern
        if let Some(set) = &self.glob_set {
            if set.is_match(filename) {
                return true;
            }
        }
 
        // check extension
        for ext in &self.extensions {
            if ext.to_lowercase() == extension {
                return true;
            }
        }
        false
    }

    pub fn sanitize(&mut self) {
        // extensions sanitize
        self.extensions = self
            .extensions
            .iter()
            .map(|ext| {
                let mut ext = ext.clone().to_lowercase();
                if ext.starts_with('.') {
                    ext = ext[1..].to_string();
                }
                ext
            })
            .collect();

        // patterns compile
        if !self.patterns.is_empty() {
            let mut builder = globset::GlobSetBuilder::new();
            for pattern in &self.patterns {
                let pattern_lower = pattern.to_lowercase();
                match globset::Glob::new(&pattern_lower) {
                    Ok(g) => {
                        builder.add(g);
                    }
                    Err(e) => {
                        tracing::warn!("Invalid pattern {:?}: {}", pattern_lower, e);
                    }
                }
            }
            self.glob_set = builder.build().ok();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub source_dir: PathBuf,
    pub rules: Vec<Rule>,
    pub watch_mode: bool,
    pub debounce_seconds: u64,
    pub polling_interval_ms: u64,
    pub temp_extensions: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub trash_dir: PathBuf,
    pub duplicates_dir: PathBuf,
    pub unknown_dir: PathBuf,
    pub history_file: String,
    pub exclude_hidden: bool,
    pub model_dir: PathBuf,
    pub ai_enabled: bool,
    pub ai_threshold: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("."),
            rules: vec![],
            watch_mode: false,
            debounce_seconds: 2,
            polling_interval_ms: 100,
            temp_extensions: vec![],
            ignore_patterns: vec![],
            trash_dir: PathBuf::from(".panos_trash"),
            duplicates_dir: PathBuf::from(".panos_duplicates"),
            unknown_dir: PathBuf::from(".panos_unknown"),
            history_file: ".history.json".to_string(),
            exclude_hidden: true,
            model_dir: PathBuf::from("model_assets"),
            ai_enabled: false,
            ai_threshold: 0.55,
        }
    }
}

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let content: String = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;

        if !config.source_dir.exists() {
            return Err(anyhow::anyhow!(
                "Source directory does not exist: {:?}",
                config.source_dir
            ));
        }

        config.sanitize();

        Ok(config)
    }

    pub fn sanitize(&mut self) {
        // temp_extensions sanitize
        self.temp_extensions = self
            .temp_extensions
            .iter()
            .map(|ext| {
                let mut ext = ext.clone().to_lowercase();
                if ext.starts_with('.') {
                    ext = ext[1..].to_string();
                }
                ext
            })
            .collect();

        // rules sanitize
        for rule in &mut self.rules {
            rule.sanitize();
        }
    }
}
