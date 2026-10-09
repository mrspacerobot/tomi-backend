//! Every list the app offers, with its words, for one language.

use crate::domain::display::Shown;
use crate::domain::ids::SavedId;
use crate::domain::language::Language;
use crate::domain::library::{Course, Equipment, Library, Skill};
use serde::Serialize;

#[cfg(test)]
mod tests;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    library_version: i64,
    diets: Vec<Shown>,
    avoid_items: Vec<Shown>,
    cuisines: Vec<Shown>,
    equipment: Vec<ShownEquipment>,
    experience: Vec<Shown>,
    units: Vec<Shown>,
    ingredients: Vec<Shown>,
    courses: Vec<ShownCourse>,
}

#[derive(Serialize)]
struct ShownEquipment {
    #[serde(flatten)]
    shown: Shown,
    /// Whether a kitchen is taken to have it until the cook answers.
    assumed: bool,
}

#[derive(Serialize)]
struct ShownCourse {
    id: SavedId,
    title: String,
    skills: Vec<ShownSkill>,
}

#[derive(Serialize)]
struct ShownSkill {
    id: SavedId,
    name: String,
    summary: String,
    prerequisites: Vec<SavedId>,
    levels: Vec<ShownLevel>,
}

#[derive(Serialize)]
struct ShownLevel {
    number: u8,
    name: String,
}

pub fn options(library: &Library, language: &Language) -> Options {
    let listed = &library.table.listed;
    let choices = &library.choices;
    Options {
        library_version: library.version,
        diets: Shown::all(&listed.diets, language),
        avoid_items: Shown::all(&listed.avoid_items, language),
        cuisines: Shown::all(&choices.cuisines, language),
        equipment: library.equipment.items.iter().map(|item| shown_equipment(item, language)).collect(),
        experience: Shown::all(&choices.experience, language),
        units: Shown::all(&choices.units, language),
        ingredients: Shown::all(&listed.ingredients, language),
        courses: library.catalog.courses.iter().map(|course| shown_course(library, course, language)).collect(),
    }
}

fn shown_equipment(item: &Equipment, language: &Language) -> ShownEquipment {
    let text = |words: &crate::domain::library::LocalizedText| words.text(language.as_str()).to_string();
    let shown = Shown { id: item.id.clone(), name: text(&item.name), summary: Some(text(&item.summary)) };
    ShownEquipment { shown, assumed: item.assumed }
}

fn shown_course(library: &Library, course: &Course, language: &Language) -> ShownCourse {
    let in_course = library.catalog.skills.iter().filter(|skill| skill.course == course.id);
    let skills = in_course.map(|skill| shown_skill(skill, language)).collect();
    ShownCourse { id: course.id.clone(), title: course.title.text(language.as_str()).to_string(), skills }
}

fn shown_skill(skill: &Skill, language: &Language) -> ShownSkill {
    let code = language.as_str();
    let levels =
        skill.levels.iter().map(|level| ShownLevel { number: level.number, name: level.name.text(code).into() });
    ShownSkill {
        id: skill.id.clone(),
        name: skill.name.text(code).to_string(),
        summary: skill.summary.text(code).to_string(),
        prerequisites: skill.prerequisites.clone(),
        levels: levels.collect(),
    }
}
