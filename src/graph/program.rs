//! The graph's program: which techniques each skill level teaches.
//!
//! A skill is named by its slug, as in Tomi's catalog, and a level by its number.
//! A program this server cannot read teaches nothing, so no lesson is matched to it by guesswork.

use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[cfg(test)]
mod tests;

const PROGRAM_PREFIX: &str = "program/";

#[derive(Deserialize)]
struct ProgramFile {
    courses: Vec<CourseFile>,
}

#[derive(Deserialize)]
struct CourseFile {
    skills: Vec<SkillFile>,
}

#[derive(Deserialize)]
struct SkillFile {
    slug: String,
    levels: Vec<LevelFile>,
}

#[derive(Deserialize)]
struct LevelFile {
    number: u8,
    teaches: Vec<Taught>,
}

#[derive(Deserialize)]
struct Taught {
    thing: String,
}

#[derive(Debug, Default)]
pub struct Program {
    taught: BTreeMap<(String, u8), Vec<String>>,
}

impl Program {
    pub fn from_things(things: &HashMap<String, Value>) -> Program {
        let mut program = Program::default();
        for (thing_id, thing) in things {
            if !thing_id.starts_with(PROGRAM_PREFIX) {
                continue;
            }
            if let Ok(file) = serde_json::from_value::<ProgramFile>(thing.clone()) {
                program.add(file);
            }
        }
        program
    }

    fn add(&mut self, file: ProgramFile) {
        for skill in file.courses.into_iter().flat_map(|course| course.skills) {
            for level in skill.levels {
                let key = (skill.slug.clone(), level.number);
                let techniques = level.teaches.into_iter().map(|taught| taught.thing);
                self.taught.entry(key).or_default().extend(techniques);
            }
        }
    }

    /// The technique ids this level teaches, none when the skill or level is not in the program.
    pub fn teaches(&self, skill: &str, level: u8) -> Vec<String> {
        let found = self.taught.get(&(skill.to_string(), level));
        found.cloned().unwrap_or_default()
    }
}
