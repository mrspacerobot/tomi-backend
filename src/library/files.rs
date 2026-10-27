//! Reads the library's files from disk at start-up.

use crate::library::LibraryFiles;
use std::fs;
use std::io;
use std::path::Path;

/// The reviewed choices, restrictions, staples, progression and rank files from `folder`.
pub fn read(folder: &Path) -> io::Result<LibraryFiles> {
    Ok(LibraryFiles {
        catalog: fs::read_to_string(folder.join("catalog.json"))?,
        restrictions: fs::read_to_string(folder.join("restrictions.json"))?,
        staples: fs::read_to_string(folder.join("staples.json"))?,
        equipment: fs::read_to_string(folder.join("equipment.json"))?,
        choices: fs::read_to_string(folder.join("choices.json"))?,
        ranks: fs::read_to_string(folder.join("ranks.json"))?,
    })
}
