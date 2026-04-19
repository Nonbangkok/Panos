use panos::Config;
use panos::rules::matcher::{find_rule_for_file, is_temp_file};
use std::path::PathBuf;

use crate::common::test_rule;

#[test]
fn test_find_rule_priority() {
    let rules = vec![
        test_rule("Docs", vec!["pdf"], vec![]),
        test_rule("Important", vec!["pdf"], vec![]),
    ];

    let result = find_rule_for_file(&PathBuf::from("report.pdf"), &rules);

    assert_eq!(result.unwrap().name, "Docs");
}

#[test]
fn test_case_insensitivity() {
    let rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    assert!(find_rule_for_file(&PathBuf::from("PHOTO.JPG"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("IMAGE.jpg"), &rules).is_some());
}

#[test]
fn test_glob_pattern_matching() {
    let rules = vec![test_rule("Reports", vec![], vec!["report_*.txt"])];

    assert!(find_rule_for_file(&PathBuf::from("report_2024.txt"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("my_report_final.txt"), &rules).is_none());
}

#[test]
fn test_no_extension_file() {
    let rules = vec![test_rule("Docs", vec!["txt"], vec![])];

    let result = find_rule_for_file(&PathBuf::from("README"), &rules);
    assert!(result.is_none());
}

#[test]
fn test_complex_is_temp_file() {
    let mut config = Config::default();
    config.temp_extensions = vec!["tmp".to_string(), "part".to_string()];
    config.sanitize();

    assert!(is_temp_file(
        &PathBuf::from(".tmp"),
        &config.temp_extensions
    ));
    assert!(!is_temp_file(
        &PathBuf::from("tmp.pdf"),
        &config.temp_extensions
    ));
}

#[test]
fn test_multiple_dots_handling() {
    let rules = vec![test_rule("Archives", vec!["gz"], vec![])];

    assert!(find_rule_for_file(&PathBuf::from("archive.tar.gz"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("my.backup.file.gz"), &rules).is_some());
}

#[test]
fn test_spaces_and_special_chars() {
    let rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    assert!(find_rule_for_file(&PathBuf::from("my vacation photo.jpg"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("image_(copy)_01.jpg"), &rules).is_some());
}

#[test]
fn test_extension_matching_robustness() {
    let rules = vec![test_rule("Docs", vec![".pdf"], vec![])];

    let result = find_rule_for_file(&PathBuf::from("test.pdf"), &rules);

    assert!(result.is_some());
}

#[test]
fn test_is_temp_file_case_insensitive() {
    let mut config = Config::default();
    config.temp_extensions = vec!["TMP".to_string()];
    config.sanitize();

    assert!(is_temp_file(
        &PathBuf::from("data.tmp"),
        &config.temp_extensions
    ));
    assert!(is_temp_file(
        &PathBuf::from("DATA.TMP"),
        &config.temp_extensions
    ));
}

#[test]
fn test_path_with_subdirectories() {
    let rules = vec![test_rule("Images", vec!["png"], vec![])];

    let path = PathBuf::from("downloads/2024/january/logo.png");
    let result = find_rule_for_file(&path, &rules);

    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Images");
}

#[test]
fn test_multiple_patterns_one_rule() {
    let rules = vec![test_rule("Media", vec![], vec!["vid_*", "movie_*"])];

    assert!(find_rule_for_file(&PathBuf::from("vid_beach.mp4"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("movie_night.mkv"), &rules).is_some());
}

#[test]
fn test_extension_or_pattern_match() {
    let rules = vec![test_rule("Docs", vec!["pdf"], vec!["manual_*"])];

    assert!(find_rule_for_file(&PathBuf::from("guide.pdf"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("manual_setup.txt"), &rules).is_some());
}

#[test]
fn test_unicode_filename_matching() {
    let rules = vec![test_rule("ThaiDocs", vec!["docx"], vec!["การบ้าน_*"])];

    assert!(find_rule_for_file(&PathBuf::from("การบ้าน_เลข.docx"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("รายงาน.docx"), &rules).is_some());
}

#[test]
fn test_dotfile_matching() {
    let rules = vec![test_rule("Config", vec!["env", "gitignore"], vec![])];

    assert!(find_rule_for_file(&PathBuf::from(".env"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from(".gitignore"), &rules).is_some());
}

#[test]
fn test_glob_character_range() {
    let rules = vec![test_rule("Numbered", vec![], vec!["img_[0-9].jpg"])];

    assert!(find_rule_for_file(&PathBuf::from("img_5.jpg"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("img_a.jpg"), &rules).is_none());
}

#[test]
fn test_glob_single_char_wildcard() {
    let rules = vec![test_rule("FourChars", vec![], vec!["????.txt"])];

    assert!(find_rule_for_file(&PathBuf::from("test.txt"), &rules).is_some());
    assert!(find_rule_for_file(&PathBuf::from("testing.txt"), &rules).is_none());
}

#[test]
fn test_empty_rule_criteria() {
    let rules = vec![test_rule("Empty", vec![], vec![])];

    assert!(find_rule_for_file(&PathBuf::from("any.file"), &rules).is_none());
}

#[test]
fn test_pattern_lowercase_consistency() {
    let rules = vec![test_rule("Docs", vec![], vec!["REPORT_*.txt"])];

    assert!(find_rule_for_file(&PathBuf::from("report_daily.txt"), &rules).is_some());
}

#[test]
fn test_extension_with_multiple_dots_complex() {
    let rules = vec![test_rule("Versions", vec!["zip"], vec![])];

    assert!(find_rule_for_file(&PathBuf::from("project.v1.0.final.backup.zip"), &rules).is_some());
}

#[test]
fn test_very_long_filename() {
    let rules = vec![test_rule("Logs", vec!["log"], vec![])];

    let long_name = "a".repeat(250) + ".log";
    assert!(find_rule_for_file(&PathBuf::from(long_name), &rules).is_some());
}
