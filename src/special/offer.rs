//! A special as the app shows it: names in the cook's language, and the
//! restrictions the server checked it against.

use super::{AppliedSwap, Candidate};
use crate::kernel::display::{Named, Namer};
use crate::kernel::ids::SavedId;
use crate::profile::Profile;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub lesson_id: SavedId,
    pub revision: u32,
    pub title: String,
    pub minutes: u32,
    pub skill: Named,
    pub level: ShownLevel,
    pub ingredients: Vec<Named>,
    pub swaps: Vec<ShownSwap>,
    pub missing: Vec<Named>,
    pub reason: Option<String>,
    pub checked: Checked,
}

#[derive(Debug, Serialize)]
pub struct ShownLevel {
    pub number: u8,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ShownSwap {
    pub removed: Named,
    pub added: Named,
}

/// The restrictions this offer was checked against, so the app can refuse
/// an offer that was checked against less than the cook asked for.
#[derive(Debug, Serialize)]
pub struct Checked {
    pub diet: Option<SavedId>,
    pub avoid: BTreeSet<SavedId>,
}

/// What every offer of one answer shares.
pub struct OfferContext<'a> {
    pub namer: Namer<'a>,
    pub profile: &'a Profile,
}

impl Offer {
    pub fn new(candidate: Candidate, reason: Option<String>, context: &OfferContext) -> Self {
        let namer = &context.namer;
        let (skill, level) = skill_and_level(&candidate, namer);
        Offer {
            title: title(&candidate, namer),
            skill,
            level,
            ingredients: namer.ingredients(&candidate.ingredients),
            swaps: candidate.swaps.iter().map(|swap| shown_swap(swap, namer)).collect(),
            missing: namer.ingredients(&candidate.missing),
            reason,
            checked: Checked { diet: context.profile.diet.clone(), avoid: context.profile.avoid.clone() },
            lesson_id: candidate.lesson_id,
            revision: candidate.revision,
            minutes: candidate.minutes,
        }
    }
}

fn title(candidate: &Candidate, namer: &Namer) -> String {
    match namer.library.lessons.get(&candidate.lesson_id) {
        Some(lesson) => lesson.title.text(namer.language.as_str()).to_string(),
        None => candidate.title.clone(),
    }
}

fn skill_and_level(candidate: &Candidate, namer: &Namer) -> (Named, ShownLevel) {
    let language = namer.language.as_str();
    let skill = namer.library.catalog.skill(&candidate.skill);
    let skill_name = skill.map(|skill| skill.name.text(language).to_string());
    let level = skill.and_then(|skill| skill.level(candidate.level));
    let level_name = level.map(|level| level.name.text(language).to_string());
    let named = Named { id: candidate.skill.clone(), name: skill_name.unwrap_or_else(|| candidate.skill.to_string()) };
    (named, ShownLevel { number: candidate.level, name: level_name.unwrap_or_default() })
}

fn shown_swap(swap: &AppliedSwap, namer: &Namer) -> ShownSwap {
    ShownSwap { removed: namer.ingredient(&swap.removed), added: namer.ingredient(&swap.added) }
}
