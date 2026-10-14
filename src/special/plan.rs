//! Planning one lesson for tonight: which listed swaps to apply, and what
//! is still missing.

use super::{restrictions, AppliedSwap, Candidate, Tonight};
use crate::kernel::ids::SavedId;
use crate::library::{Lesson, Library, Restrictions};

/// The lesson as it would be cooked tonight, or `None` when no listed swap
/// makes it safe for this cook.
pub fn cook(library: &Library, lesson: &Lesson, tonight: &Tonight) -> Option<Candidate> {
    let planner = Planner { library, lesson, tonight, cook: restrictions(tonight.profile) };
    let mut planned = Planned::default();
    for original in lesson.ingredient_ids() {
        let chosen = planner.choose(original)?;
        planned.add(original, chosen);
    }
    // The final check, on the whole dish as it will be cooked.
    if !planned.ingredients.iter().all(|id| planner.allowed(id)) {
        return None;
    }
    Some(planned.into_candidate(lesson, tonight))
}

struct Planner<'a> {
    library: &'a Library,
    lesson: &'a Lesson,
    tonight: &'a Tonight<'a>,
    cook: Restrictions<'a>,
}

impl Planner<'_> {
    fn allowed(&self, ingredient: &SavedId) -> bool {
        self.library.table.check(ingredient, &self.cook).is_ok()
    }

    fn at_hand(&self, ingredient: &SavedId) -> bool {
        self.tonight.available.contains(ingredient)
    }

    /// The ingredient to use for `original`: itself when allowed and at
    /// hand; else an allowed listed swap, preferring one at hand; else
    /// itself when allowed but missing. `None` when nothing allowed is left.
    fn choose(&self, original: &SavedId) -> Option<SavedId> {
        let original_allowed = self.allowed(original);
        if original_allowed && self.at_hand(original) {
            return Some(original.clone());
        }
        let swaps = self.allowed_swaps(original);
        if let Some(swap) = swaps.iter().find(|id| self.at_hand(id)) {
            return Some(swap.clone());
        }
        if original_allowed {
            return Some(original.clone());
        }
        swaps.into_iter().next()
    }

    fn allowed_swaps(&self, original: &SavedId) -> Vec<SavedId> {
        let listed = self.lesson.swaps_for(original);
        listed.filter(|id| self.allowed(id)).cloned().collect()
    }
}

#[derive(Default)]
struct Planned {
    ingredients: Vec<SavedId>,
    swaps: Vec<AppliedSwap>,
}

impl Planned {
    fn add(&mut self, original: &SavedId, chosen: SavedId) {
        if &chosen != original {
            self.swaps.push(AppliedSwap { removed: original.clone(), added: chosen.clone() });
        }
        self.ingredients.push(chosen);
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
