//! Reads the library's files from disk at start-up.

use crate::library::LibraryFiles;
use std::fs;
use std::io;
use std::path::Path;

const LESSON_ENDING: &str = ".lesson.json";

/// The catalog, restrictions and equipment list from `folder`, and every
/// `*.lesson.json` in `folder/lessons`, sorted by name.
pub fn read(folder: &Path) -> io::Result<LibraryFiles> {
    Ok(LibraryFiles {
        catalog: fs::read_to_string(folder.join("catalog.json"))?,
        restrictions: fs::read_to_string(folder.join("restrictions.json"))?,
        staples: fs::read_to_string(folder.join("staples.json"))?,
        equipment: fs::read_to_string(folder.join("equipment.json"))?,
        choices: fs::read_to_string(folder.join("choices.json"))?,
        lessons: read_lessons(&folder.join("lessons"))?,
    })
}

fn read_lessons(folder: &Path) -> io::Result<Vec<(String, String)>> {
    let mut lessons = Vec::new();
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_string();
        if name.ends_with(LESSON_ENDING) {
            lessons.push((name, fs::read_to_string(&path)?));
        }
    }
    lessons.sort();
    Ok(lessons)
}
