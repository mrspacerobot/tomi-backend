//! A graph recipe with its original content and the graph's safe plan.

use super::{AppliedSwap, Candidate};
use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::library::LocalizedText;
use crate::profile::Profile;
use serde::Serialize;
use serde_json::Value;

mod recipe;
use recipe::{AdaptedRecipe, RecipeChanges};
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub recipe_id: String,
    pub title: String,
    pub source_recipe: Value,
    /// The recipe written for this cook, in code, and checked again against their restrictions.
    pub recipe: AdaptedRecipe,
    pub changes: RecipeChanges,
    pub skill: SuggestionSkill,
    pub level: NamedLevel,
    pub safe_plan: SafePlan,
    pub reason: Option<String>,
    pub checked: CheckedRestrictions,
}

#[derive(Debug, Serialize)]
pub struct SuggestionSkill {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct NamedLevel {
    pub number: u8,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafePlan {
    pub ingredients: Vec<NamedOption>,
    pub substitutions: Vec<SuggestionSwap>,
    pub omitted_parts: Vec<String>,
    pub missing: Vec<NamedOption>,
    pub equipment: Vec<NamedOption>,
}

#[derive(Debug, Serialize)]
pub struct NamedOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SuggestionSwap {
    pub removed: NamedOption,
    pub added: NamedOption,
    pub ratio: Option<crate::graph::serving::SwapRatio>,
    pub caveat: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckedRestrictions {
    pub diet: Option<SavedId>,
    pub avoid: BTreeSet<SavedId>,
}

/// What every offer of one answer shares.
pub struct SuggestionContext<'a> {
    pub graph: &'a Graph,
    pub profile: &'a Profile,
    pub language: &'a Language,
}

impl Suggestion {
    pub fn new(candidate: Candidate, reason: Option<String>, context: &SuggestionContext) -> Option<Self> {
        let (skill, level) = suggestion_skill_and_level(&candidate, context)?;
        let safe_plan = safe_plan(&candidate, context.graph, context.language)?;
        let (recipe, changes) = recipe::adaptation(&candidate, context.graph, context.language)?;
        Some(Suggestion {
            title: localized(&candidate.title, context.language),
            recipe_id: candidate.recipe_id,
            source_recipe: candidate.source_recipe,
            recipe,
            changes,
            skill,
            level,
            safe_plan,
            reason,
            checked: CheckedRestrictions { diet: context.profile.diet.clone(), avoid: context.profile.avoid.clone() },
        })
    }
}

/// The skill and level the recipe teaches, named for the cook.
fn suggestion_skill_and_level(
    candidate: &Candidate,
    context: &SuggestionContext,
) -> Option<(SuggestionSkill, NamedLevel)> {
    let skill = context.graph.program.skill(candidate.skill.as_str())?;
    let level = context.graph.program.level(candidate.skill.as_str(), candidate.level)?;
    let skill_name = localized(skill.name.as_ref()?, context.language);
    let level_name = localized(level.name.as_ref()?, context.language);
    Some((
        SuggestionSkill { id: candidate.skill.to_string(), name: skill_name },
        NamedLevel { number: candidate.level, name: level_name },
    ))
}

fn safe_plan(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<SafePlan> {
    Some(SafePlan {
        ingredients: named_options(&candidate.ingredients, graph, language)?,
        substitutions: suggestion_swaps(&candidate.swaps, graph, language)?,
        omitted_parts: candidate.omitted.clone(),
        missing: named_options(&candidate.missing, graph, language)?,
        equipment: named_options(&candidate.equipment, graph, language)?,
    })
}

fn named_options(ids: &[String], graph: &Graph, language: &Language) -> Option<Vec<NamedOption>> {
    ids.iter().map(|id| named_option(id, graph, language)).collect()
}

fn suggestion_swaps(swaps: &[AppliedSwap], graph: &Graph, language: &Language) -> Option<Vec<SuggestionSwap>> {
    swaps.iter().map(|swap| suggestion_swap(swap, graph, language)).collect()
}

fn suggestion_swap(swap: &AppliedSwap, graph: &Graph, language: &Language) -> Option<SuggestionSwap> {
    Some(SuggestionSwap {
        removed: named_option(&swap.removed, graph, language)?,
        added: named_option(&swap.added, graph, language)?,
        ratio: swap.ratio.clone(),
        caveat: swap.caveat.clone(),
    })
}

fn named_option(id: &str, graph: &Graph, language: &Language) -> Option<NamedOption> {
    let names = graph.thing(id)?.get("names")?.clone();
    let words: LocalizedText = serde_json::from_value(names).ok()?;
    Some(NamedOption { id: id.to_string(), name: localized(&words, language) })
}

fn localized(words: &LocalizedText, language: &Language) -> String {
    words.text(language.as_str()).to_string()
}
