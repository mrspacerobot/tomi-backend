//! The rules of AGENTS.md that a test can check, so they are not hopes.

#![expect(
    clippy::unwrap_used,
    reason = "a failed unwrap in a test is a failed test, not a crash"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// A file longer than this has more than one job. See AGENTS.md.
const MAX_FILE_LINES: usize = 200;

/// Words that mean a file in `src/domain` reaches for the web, the
/// database or the clock. Rules take what they need as arguments.
const FORBIDDEN_IN_DOMAIN: [&str; 9] = [
    "axum",
    "sqlx",
    "reqwest",
    "tokio",
    "tower",
    "SystemTime::now",
    "Instant::now",
    "Utc::now",
    "Local::now",
];

#[test]
fn no_file_is_longer_than_the_limit() {
    let mut too_long = Vec::new();
    for file in rust_files(Path::new("src"))
        .into_iter()
        .chain(rust_files(Path::new("tests")))
    {
        let lines = read(&file).lines().count();
        if lines > MAX_FILE_LINES {
            too_long.push(format!("{} has {lines} lines", file.display()));
        }
    }
    assert!(too_long.is_empty(), "split these files: {too_long:#?}");
}

#[test]
fn domain_rules_need_no_web_database_or_clock() {
    let mut found = Vec::new();
    for file in rust_files(Path::new("src/domain")) {
        let text = read(&file);
        for word in FORBIDDEN_IN_DOMAIN
            .iter()
            .filter(|word| text.contains(*word))
        {
            found.push(format!("{} uses {word}", file.display()));
        }
    }
    assert!(found.is_empty(), "move this out of src/domain: {found:#?}");
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
