//! Every list the app offers, with its words, for one language.

use crate::graph::Foods;
use crate::kernel::display::Shown;
use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::library::{Choice, Course, Equipment, Kind, Library, Skill};
use serde::Serialize;

mod routes;
#[cfg(test)]
mod tests;

pub use routes::router;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    library_version: i64,
    diets: Vec<Shown>,
    avoid_items: Vec<ShownAvoidItem>,
    cuisines: Vec<Shown>,
    equipment: Vec<ShownEquipment>,
    experience: Vec<Shown>,
    units: Vec<Shown>,
    staples: Vec<ShownStaple>,
    ingredients: Vec<Shown>,
    courses: Vec<ShownCourse>,
}

/// An avoid item, with whether it is about an allergen or a choice, so the app can group them.
#[derive(Serialize)]
struct ShownAvoidItem {
    #[serde(flatten)]
    shown: Shown,
    kind: Kind,
}

/// An everyday ingredient the cupboard question offers, named by the graph.
#[derive(Serialize)]
struct ShownStaple {
    id: SavedId,
    name: String,
    /// Counted as at home until the cook has answered.
    assumed: bool,
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

pub fn options(library: &Library, foods: &Foods, language: &Language) -> Options {
    let restrictions = &library.restrictions;
    let choices = &library.choices;
    Options {
        library_version: library.version,
        diets: Shown::all(&restrictions.diets, language),
        avoid_items: shown_avoid_items(library, language),
        cuisines: Shown::all(&choices.cuisines, language),
        equipment: library.equipment.items.iter().map(|item| shown_equipment(item, language)).collect(),
        experience: Shown::all(&choices.experience, language),
        units: Shown::all(&choices.units, language),
        staples: shown_staples(library, foods, language),
        ingredients: Shown::all(foods.listed(), language),
        courses: library.catalog.courses.iter().map(|course| shown_course(library, course, language)).collect(),
    }
}

fn shown_avoid_items(library: &Library, language: &Language) -> Vec<ShownAvoidItem> {
    let shown = |choice: &Choice| ShownAvoidItem {
        shown: Shown::of(choice, language),
        kind: library.restrictions.kind_of(&choice.id),
    };
    library.restrictions.avoid_items.iter().map(shown).collect()
}

/// The staples the graph can name. Without a graph there are no names, so the list is empty rather than guessed.
fn shown_staples(library: &Library, foods: &Foods, language: &Language) -> Vec<ShownStaple> {
    let mut shown = Vec::new();
    for staple in library.staples.listed() {
        if let Some(choice) = foods.choice(&staple.ingredient) {
            let name = choice.name.text(language.as_str()).to_string();
            shown.push(ShownStaple { id: staple.ingredient.clone(), name, assumed: staple.assumed });
        }
    }
    shown
}

fn shown_equipment(item: &Equipment, language: &Language) -> ShownEquipment {
    let text = |words: &crate::library::LocalizedText| words.text(language.as_str()).to_string();
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
