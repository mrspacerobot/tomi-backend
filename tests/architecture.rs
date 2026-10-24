//! The rules of AGENTS.md that a test can check, so they are not hopes.

#![expect(clippy::unwrap_used, reason = "a failed unwrap in a test is a failed test, not a crash")]

use std::fs;
use std::path::{Path, PathBuf};

/// A file longer than this has more than one job. See AGENTS.md.
const MAX_FILE_LINES: usize = 200;

/// Words that mean a file reaches for the web, the database or the clock. The rules take what they
/// need as arguments.
const FORBIDDEN_IN_RULES: [&str; 9] =
    ["axum", "sqlx", "reqwest", "tokio", "tower", "SystemTime::now", "Instant::now", "Utc::now", "Local::now"];

/// The features whose files are pure rules, except the ones in `TALKS_TO_THE_OUTSIDE`.
const FEATURES: [&str; 9] =
    ["graph", "ingredients", "kernel", "library", "options", "profile", "recipe", "scans", "suggestions"];

/// The files of a feature that talk to the web, the database, the disk or storage on purpose.
const TALKS_TO_THE_OUTSIDE: [&str; 4] = ["routes.rs", "db.rs", "files.rs", "keep.rs"];

/// Everything else under `src` is plumbing that may use the web, the database or the clock.
const PLUMBING: [&str; 10] = ["ai", "auth", "config", "db", "error", "http", "lib", "main", "meta", "storage"];

#[test]
fn no_file_is_longer_than_the_limit() {
    let mut too_long = Vec::new();
    for file in rust_files(Path::new("src")).into_iter().chain(rust_files(Path::new("tests"))) {
        let lines = read(&file).lines().count();
        if lines > MAX_FILE_LINES {
            too_long.push(format!("{} has {lines} lines", file.display()));
        }
    }
    assert!(too_long.is_empty(), "split these files: {too_long:#?}");
}

#[test]
fn the_rules_of_each_feature_need_no_web_database_or_clock() {
    let mut found = Vec::new();
    for file in rule_files() {
        let text = read(&file);
        for word in FORBIDDEN_IN_RULES.iter().filter(|word| text.contains(*word)) {
            found.push(format!("{} uses {word}", file.display()));
        }
    }
    assert!(found.is_empty(), "move this out of the rules, into routes.rs, db.rs or files.rs: {found:#?}");
}

#[test]
fn every_folder_under_src_is_a_feature_or_plumbing() {
    let mut unknown = Vec::new();
    for entry in fs::read_dir("src").unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        if !FEATURES.contains(&name.as_str()) && !PLUMBING.contains(&name.as_str()) {
            unknown.push(name);
        }
    }
    unknown.dedup();
    assert!(unknown.is_empty(), "list each in FEATURES (pure rules) or PLUMBING: {unknown:?}");
}

/// The feature's root file and every file below it that is not meant to talk to the outside.
fn rule_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for feature in FEATURES {
        files.push(PathBuf::from(format!("src/{feature}.rs")));
        let inside = rust_files(&PathBuf::from(format!("src/{feature}")));
        files.extend(inside.into_iter().filter(|file| !talks_to_the_outside(file)));
    }
    files
}

fn talks_to_the_outside(file: &Path) -> bool {
    let name = file.file_name().unwrap().to_string_lossy();
    TALKS_TO_THE_OUTSIDE.contains(&name.as_ref())
}

fn read(file: &Path) -> String {
    fs::read_to_string(file).unwrap()
}

fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}
