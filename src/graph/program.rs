//! The graph's program: which techniques each skill level teaches.
//!
//! A skill is named by its slug, as in Tomi's catalog, and a level by its number.
//! A program this server cannot read teaches nothing, so no lesson is matched to it by guesswork.

use crate::library::LocalizedText;
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
    slug: String,
    #[serde(default)]
    names: Option<LocalizedText>,
    skills: Vec<SkillFile>,
}

#[derive(Deserialize)]
struct SkillFile {
    slug: String,
    #[serde(default)]
    names: Option<LocalizedText>,
    #[serde(default)]
    comes_after: Vec<Prerequisite>,
    levels: Vec<LevelFile>,
}

#[derive(Deserialize)]
struct LevelFile {
    number: u8,
    #[serde(default)]
    names: Option<LocalizedText>,
    teaches: Vec<Taught>,
    #[serde(default)]
    through: Vec<Taught>,
}

#[derive(Deserialize)]
struct Taught {
    thing: String,
}

#[derive(Deserialize)]
struct Prerequisite {
    skill: String,
}

#[derive(Debug, Default)]
pub struct Program {
    taught: BTreeMap<(String, u8), Vec<String>>,
    variants: BTreeMap<String, Vec<String>>,
    courses: Vec<ProgramCourse>,
}

#[derive(Debug, Clone)]
pub struct ProgramCourse {
    pub id: String,
    pub name: Option<LocalizedText>,
    pub skills: Vec<ProgramSkill>,
}

#[derive(Debug, Clone)]
pub struct ProgramSkill {
    pub id: String,
    pub course: String,
    pub name: Option<LocalizedText>,
    pub prerequisites: Vec<String>,
    pub levels: Vec<ProgramLevel>,
}

#[derive(Debug, Clone)]
pub struct ProgramLevel {
    pub number: u8,
    pub name: Option<LocalizedText>,
    pub teaches: Vec<String>,
    pub through: Vec<String>,
}

impl Program {
    pub fn from_things(things: &HashMap<String, Value>) -> Program {
        let mut program = Program::default();
        for (thing_id, thing) in things {
            program.add_variants(thing_id, thing);
            if !thing_id.starts_with(PROGRAM_PREFIX) {
                continue;
            }
            if let Ok(file) = serde_json::from_value::<ProgramFile>(thing.clone()) {
                program.add(file);
            }
        }
        program
    }

    /// Skill levels that teach a technique used by this recipe, including direct graph variants.
    pub fn levels_for(&self, techniques: &[String]) -> Vec<(String, u8)> {
        let mut found = Vec::new();
        for ((skill, level), taught) in &self.taught {
            if taught.iter().any(|technique| self.is_used(technique, techniques)) {
                found.push((skill.clone(), *level));
            }
        }
        found
    }

    pub fn courses(&self) -> &[ProgramCourse] {
        &self.courses
    }

    pub fn skill(&self, id: &str) -> Option<&ProgramSkill> {
        self.courses.iter().flat_map(|course| &course.skills).find(|skill| skill.id == id)
    }

    pub fn level(&self, skill_id: &str, number: u8) -> Option<&ProgramLevel> {
        self.skill(skill_id)?.levels.iter().find(|level| level.number == number)
    }

    fn add_variants(&mut self, thing_id: &str, thing: &Value) {
        if !thing_id.starts_with("technique/") {
            return;
        }
        let Some(variants) = thing.get("variant_of").and_then(Value::as_array) else {
            return;
        };
        for variant in variants {
            let Some(parent) = variant.get("thing").and_then(Value::as_str) else {
                continue;
            };
            self.variants.entry(parent.to_string()).or_default().push(thing_id.to_string());
        }
    }

    fn is_used(&self, taught: &str, used: &[String]) -> bool {
        let mut pending = vec![taught.to_string()];
        let mut seen = BTreeMap::new();
        while let Some(technique) = pending.pop() {
            if used.iter().any(|used| used == &technique) {
                return true;
            }
            if seen.insert(technique.clone(), ()).is_some() {
                continue;
            }
            let Some(variants) = self.variants.get(&technique) else {
                continue;
            };
            pending.extend(variants.iter().cloned());
        }
        false
    }

    fn add(&mut self, file: ProgramFile) {
        for course in file.courses {
            self.add_course(course);
        }
    }

    fn add_course(&mut self, course: CourseFile) {
        let mut skills = Vec::new();
        for skill in course.skills {
            skills.push(self.add_skill(&course.slug, skill));
        }
        self.courses.push(ProgramCourse { id: course.slug, name: course.names, skills });
    }

    fn add_skill(&mut self, course: &str, skill: SkillFile) -> ProgramSkill {
        let prerequisites = skill.comes_after.into_iter().map(|item| item.skill).collect();
        let levels = skill.levels.into_iter().map(|level| self.add_level(&skill.slug, level)).collect();
        ProgramSkill { id: skill.slug, course: course.to_string(), name: skill.names, prerequisites, levels }
    }

    fn add_level(&mut self, skill: &str, level: LevelFile) -> ProgramLevel {
        let teaches: Vec<String> = level.teaches.into_iter().map(|item| item.thing).collect();
        self.taught.entry((skill.to_string(), level.number)).or_default().extend(teaches.iter().cloned());
        ProgramLevel {
            number: level.number,
            name: level.names,
            teaches,
            through: level.through.into_iter().map(|item| item.thing).collect(),
        }
    }

    /// The technique ids this level teaches, none when the skill or level is not in the program.
    pub fn teaches(&self, skill: &str, level: u8) -> Vec<String> {
        let found = self.taught.get(&(skill.to_string(), level));
        found.cloned().unwrap_or_default()
    }
}
