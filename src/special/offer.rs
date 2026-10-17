//! A graph recipe with its original content and the graph's safe plan.

use super::{AppliedSwap, Candidate};
use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::library::LocalizedText;
use crate::profile::Profile;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub recipe_id: String,
    pub title: String,
    pub source_recipe: Value,
    pub skill: ShownSkill,
    pub level: ShownLevel,
    pub safe_plan: SafePlan,
    pub reason: Option<String>,
    pub checked: Checked,
}

#[derive(Debug, Serialize)]
pub struct ShownSkill {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ShownLevel {
    pub number: u8,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafePlan {
    pub ingredients: Vec<NamedThing>,
    pub substitutions: Vec<ShownSwap>,
    pub omitted_parts: Vec<String>,
    pub missing: Vec<NamedThing>,
    pub equipment: Vec<NamedThing>,
}

#[derive(Debug, Serialize)]
pub struct NamedThing {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ShownSwap {
    pub removed: NamedThing,
    pub added: NamedThing,
    pub ratio: Option<crate::graph::serving::Ratio>,
    pub caveat: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Checked {
    pub diet: Option<SavedId>,
    pub avoid: BTreeSet<SavedId>,
}

/// What every offer of one answer shares.
pub struct OfferContext<'a> {
    pub graph: &'a Graph,
    pub profile: &'a Profile,
    pub language: &'a Language,
}

impl Offer {
    pub fn new(candidate: Candidate, reason: Option<String>, context: &OfferContext) -> Option<Self> {
        let skill = context.graph.program.skill(candidate.skill.as_str())?;
        let level = context.graph.program.level(candidate.skill.as_str(), candidate.level)?;
        let skill_name = localized(skill.name.as_ref()?, context.language);
        let level_name = localized(level.name.as_ref()?, context.language);
        let safe_plan = safe_plan(&candidate, context.graph, context.language)?;
        Some(Offer {
            recipe_id: candidate.recipe_id,
            title: localized(&candidate.title, context.language),
            source_recipe: candidate.source_recipe,
            skill: ShownSkill { id: candidate.skill.to_string(), name: skill_name },
            level: ShownLevel { number: candidate.level, name: level_name },
            safe_plan,
            reason,
            checked: Checked { diet: context.profile.diet.clone(), avoid: context.profile.avoid.clone() },
        })
    }
}

fn safe_plan(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<SafePlan> {
    Some(SafePlan {
        ingredients: named_things(&candidate.ingredients, graph, language)?,
        substitutions: shown_swaps(&candidate.swaps, graph, language)?,
        omitted_parts: candidate.omitted.clone(),
        missing: named_things(&candidate.missing, graph, language)?,
        equipment: named_things(&candidate.equipment, graph, language)?,
    })
}

fn named_things(ids: &[String], graph: &Graph, language: &Language) -> Option<Vec<NamedThing>> {
    ids.iter().map(|id| named_thing(id, graph, language)).collect()
}

fn shown_swaps(swaps: &[AppliedSwap], graph: &Graph, language: &Language) -> Option<Vec<ShownSwap>> {
    swaps.iter().map(|swap| shown_swap(swap, graph, language)).collect()
}

fn shown_swap(swap: &AppliedSwap, graph: &Graph, language: &Language) -> Option<ShownSwap> {
    Some(ShownSwap {
        removed: named_thing(&swap.removed, graph, language)?,
        added: named_thing(&swap.added, graph, language)?,
        ratio: swap.ratio.clone(),
        caveat: swap.caveat.clone(),
    })
}

fn named_thing(id: &str, graph: &Graph, language: &Language) -> Option<NamedThing> {
    let names = graph.thing(id)?.get("names")?.clone();
    let words: LocalizedText = serde_json::from_value(names).ok()?;
    Some(NamedThing { id: id.to_string(), name: localized(&words, language) })
}

fn localized(words: &LocalizedText, language: &Language) -> String {
    words.text(language.as_str()).to_string()
}
