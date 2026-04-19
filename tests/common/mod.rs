use panos::{Config, Rule};
use std::path::{Path, PathBuf};

pub fn test_config(root: &Path) -> Config {
    Config {
        source_dir: root.to_path_buf(),
        ..Config::default()
    }
}

pub fn test_rule(name: &str, exts: Vec<&str>, patterns: Vec<&str>) -> Rule {
    let mut rule = Rule {
        name: name.to_string(),
        extensions: exts.into_iter().map(|s| s.to_string()).collect(),
        patterns: patterns.iter().map(|s| s.to_string()).collect(),
        destination: PathBuf::from(name),
        ..Rule::default()
    };
    rule.sanitize();
    rule
}
