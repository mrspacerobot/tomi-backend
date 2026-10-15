//! Planning one lesson for tonight: the graph's plan for its recipe, as the lesson's own ingredients.
//!
//! The graph decides what is safe and which swaps are allowed. Here that answer is read back onto the lesson.

use super::links::recipe_for;
use super::{AppliedSwap, Candidate, Tonight};
use crate::graph::serving::{CookFacts, RecipePlan, ServedRecipe};
use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::library::Lesson;

pub const INGREDIENT_PREFIX: &str = "ingredient/";

/// What a lesson is planned against: the graph, what this cook avoids and has, and the rest of tonight.
pub struct Kitchen<'a> {
    pub graph: &'a Graph,
    pub facts: &'a CookFacts,
    pub tonight: &'a Tonight<'a>,
}

/// The lesson as it would be cooked tonight, or `None` when the graph finds no safe way to make its recipe.
pub fn cook(lesson: &Lesson, kitchen: &Kitchen) -> Option<Candidate> {
    let recipe_id = recipe_for(lesson, kitchen.graph).ok()?;
    let plan = match kitchen.graph.serving.plan(&recipe_id, kitchen.facts) {
        Ok(Some(plan)) => plan,
        _ => return None,
    };
    let recipe = kitchen.graph.serving.recipes.get(&recipe_id)?;
    let planned = Planned::read(lesson, recipe, &plan)?;
    Some(planned.into_candidate(lesson, kitchen.tonight))
}

#[derive(Default)]
struct Planned {
    ingredients: Vec<SavedId>,
    swaps: Vec<AppliedSwap>,
}

impl Planned {
    /// The lesson's ingredients after the plan's swaps, without the optional parts it leaves out. `None` when a swap
    /// is something this server cannot name as an ingredient: better no offer than a wrong one.
    fn read(lesson: &Lesson, recipe: &ServedRecipe, plan: &RecipePlan) -> Option<Planned> {
        let mut planned = Planned::default();
        for original in lesson.ingredient_ids() {
            let thing = format!("{INGREDIENT_PREFIX}{original}");
            let line = recipe.lines.iter().find(|line| line.thing == thing)?;
            if plan.left_out.contains(&line.part) {
                continue;
            }
            match plan.swaps.iter().find(|swap| swap.original == thing) {
                Some(swap) => planned.swap(original, &swap.instead)?,
                None => planned.ingredients.push(original.clone()),
            }
        }
        Some(planned)
    }

    fn swap(&mut self, original: &SavedId, instead: &str) -> Option<()> {
        let slug = instead.strip_prefix(INGREDIENT_PREFIX)?;
        let added = SavedId::parse(slug).ok()?;
        self.swaps.push(AppliedSwap { removed: original.clone(), added: added.clone() });
        self.ingredients.push(added);
        Some(())
    }

    fn into_candidate(self, lesson: &Lesson, tonight: &Tonight) -> Candidate {
        let missing = self.ingredients.iter().filter(|id| !tonight.available.contains(*id));
        Candidate {
            lesson_id: lesson.id.clone(),
            revision: lesson.revision,
            title: lesson.title.english().to_string(),
            minutes: lesson.minutes,
            skill: lesson.skill.clone(),
            level: lesson.level,
            missing: missing.cloned().collect(),
            ingredients: self.ingredients,
            swaps: self.swaps,
        }
    }
}
