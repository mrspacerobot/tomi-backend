use super::*;
use crate::graph::serving::{PlannedSwap, Ratio};
use crate::graph::steps::StepError;
use crate::graph::test_helpers::{change_thing, fixture_graph};
use serde_json::json;

const EGGS: &str = "recipe/soft-scrambled-eggs";
const PAN_SAUCE: &str = "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce";

fn swap(part: &str, original: &str, instead: &str) -> PlannedSwap {
    let (part, original, instead) = (part.to_string(), original.to_string(), instead.to_string());
    PlannedSwap { part, original, instead, ratio: None, caveat: None }
}

fn plan(swaps: Vec<PlannedSwap>, left_out: &[&str]) -> RecipePlan {
    RecipePlan { swaps, left_out: left_out.iter().map(|part| part.to_string()).collect() }
}

fn avoiding(facts: &[&str]) -> BTreeSet<String> {
    facts.iter().map(|fact| fact.to_string()).collect()
}

fn adapted(graph: &Graph, recipe: &str, plan: &RecipePlan) -> Result<Adapted, AdaptError> {
    adapt(recipe, plan, &Basis { graph, avoided: &BTreeSet::new() })
}

fn texts(adapted: &Adapted) -> Vec<&str> {
    adapted.steps.iter().map(|step| step.text.as_str()).collect()
}

fn cream_for_creme_fraiche() -> RecipePlan {
    plan(vec![swap("enrichment", "ingredient/cream", "ingredient/creme-fraiche")], &[])
}

#[test]
fn a_recipe_with_no_changes_is_written_with_its_own_foods() {
    let made = adapted(&fixture_graph(), EGGS, &RecipePlan::default()).unwrap();
    assert!(texts(&made)[0].starts_with("Crack the eggs into a bowl, add the salt"));
    assert!(texts(&made)[2].contains("stir in the cold butter and the cream to stop"));
    assert_eq!(made.lines.len(), 4);
}

#[test]
fn a_swap_is_written_into_every_step_that_used_the_food() {
    let made = adapted(
        &fixture_graph(),
        PAN_SAUCE,
        &plan(vec![swap("sunflower-oil", "ingredient/sunflower-oil", "ingredient/olive-oil")], &[]),
    )
    .unwrap();
    let mentions = texts(&made).iter().filter(|text| text.contains("olive oil")).count();
    assert_eq!(mentions, 1);
    let made = adapted(&fixture_graph(), EGGS, &cream_for_creme_fraiche()).unwrap();
    assert!(texts(&made)[2].contains("stir in the cold butter and the crème fraîche to stop"));
    assert!(texts(&made).iter().all(|text| !text.contains("cream")));
    assert!(made.lines.iter().any(|line| line.food == "ingredient/creme-fraiche"));
    assert!(made.lines.iter().all(|line| line.food != "ingredient/cream"));
}

#[test]
fn a_swap_is_listed_with_its_caveat_and_ratio() {
    let ratio = Ratio { stand_in: 1.0, original: 1.0, by: "volume".into() };
    let wine = PlannedSwap {
        ratio: Some(ratio),
        caveat: Some("less sharp".into()),
        ..swap("liquid", "ingredient/white-wine", "ingredient/chicken-stock")
    };
    let made = adapted(&fixture_graph(), PAN_SAUCE, &plan(vec![wine.clone()], &[])).unwrap();
    assert_eq!(made.swaps, vec![wine]);
    assert!(texts(&made).iter().any(|text| text.starts_with("Pour in the chicken stock")));
}

#[test]
fn a_part_left_out_loses_its_clauses_and_the_steps_that_were_only_about_it() {
    let graph = fixture_graph();
    let kept = adapted(&graph, PAN_SAUCE, &RecipePlan::default()).unwrap();
    let without = adapted(&graph, PAN_SAUCE, &plan(vec![], &["spinach"])).unwrap();
    assert_eq!(kept.steps.len(), 9);
    assert_eq!(without.steps.len(), 8);
    assert_eq!(texts(&without)[7], "Plate the chicken, skin side up, and spoon the sauce around it.");
    assert!(texts(&kept).contains(&"Plate the spinach with the chicken, skin side up, and spoon the sauce around it."));
}

#[test]
fn a_part_left_out_leaves_no_line_no_number_gap_and_is_listed() {
    let graph = fixture_graph();
    let without = adapted(&graph, PAN_SAUCE, &plan(vec![], &["spinach"])).unwrap();
    assert!(texts(&without).iter().all(|text| !text.contains("pinach")));
    assert!(without.lines.iter().all(|line| line.part != "spinach"));
    let numbers: Vec<usize> = without.steps.iter().map(|step| step.number).collect();
    assert_eq!(numbers, (1..=8).collect::<Vec<_>>());
    assert_eq!(
        without.left_out,
        vec![LeftOutPart { part: "spinach".into(), foods: vec!["ingredient/spinach".into()] }]
    );
}

#[test]
fn a_ratio_scales_the_amount_and_a_missing_ratio_leaves_it() {
    let graph = fixture_graph();
    let oil = |ratio: Option<Ratio>| PlannedSwap {
        ratio,
        ..swap("sunflower-oil", "ingredient/sunflower-oil", "ingredient/olive-oil")
    };
    let amount_of =
        |made: &Adapted| made.lines.iter().find(|line| line.part == "sunflower-oil").unwrap().amount.clone();
    let own = amount_of(&adapted(&graph, PAN_SAUCE, &RecipePlan::default()).unwrap());
    let half = Some(Ratio { stand_in: 1.0, original: 2.0, by: "volume".into() });
    let scaled = amount_of(&adapted(&graph, PAN_SAUCE, &plan(vec![oil(half)], &[])).unwrap());
    assert_eq!(scaled, Amount { value: own.value / 2.0, unit: own.unit });
    assert_eq!(amount_of(&adapted(&graph, PAN_SAUCE, &plan(vec![oil(None)], &[])).unwrap()), own);
}

#[test]
fn a_ratio_that_measures_the_line_another_way_is_refused() {
    let ratio = Some(Ratio { stand_in: 2.0, original: 1.0, by: "weight".into() });
    let oil = PlannedSwap { ratio, ..swap("sunflower-oil", "ingredient/sunflower-oil", "ingredient/olive-oil") };
    let found = adapted(&fixture_graph(), PAN_SAUCE, &plan(vec![oil], &[]));
    assert_eq!(found.unwrap_err(), AdaptError::Ratio);
}

#[test]
fn a_ratio_of_one_changes_no_amount_however_it_is_measured() {
    let ratio = Some(Ratio { stand_in: 1.0, original: 1.0, by: "weight".into() });
    let oil = PlannedSwap { ratio, ..swap("sunflower-oil", "ingredient/sunflower-oil", "ingredient/olive-oil") };
    let own = adapted(&fixture_graph(), PAN_SAUCE, &RecipePlan::default()).unwrap();
    let made = adapted(&fixture_graph(), PAN_SAUCE, &plan(vec![oil], &[])).unwrap();
    assert_eq!(made.lines[2].amount, own.lines[2].amount);
}

#[test]
fn a_ratio_that_is_not_a_positive_number_is_refused() {
    let ratio = Some(Ratio { stand_in: 1.0, original: 0.0, by: "volume".into() });
    let oil = PlannedSwap { ratio, ..swap("sunflower-oil", "ingredient/sunflower-oil", "ingredient/olive-oil") };
    let found = adapted(&fixture_graph(), PAN_SAUCE, &plan(vec![oil], &[]));
    assert_eq!(found.unwrap_err(), AdaptError::Ratio);
}

#[test]
fn the_adapted_recipe_is_checked_again_against_what_the_cook_avoids() {
    let graph = fixture_graph();
    let milk = avoiding(&["fact/milk"]);
    let basis = Basis { graph: &graph, avoided: &milk };
    let swapped_into_milk = adapt(EGGS, &cream_for_creme_fraiche(), &basis);
    assert_eq!(swapped_into_milk.unwrap_err(), AdaptError::Unsafe);
    let kept_milk = adapt(EGGS, &RecipePlan::default(), &basis);
    assert_eq!(kept_milk.unwrap_err(), AdaptError::Unsafe);
}

#[test]
fn a_food_the_graph_does_not_list_for_the_line_is_refused() {
    let tofu = plan(vec![swap("egg", "ingredient/egg", "ingredient/tofu")], &[]);
    assert_eq!(adapted(&fixture_graph(), EGGS, &tofu).unwrap_err(), AdaptError::UnlistedFood);
}

#[test]
fn only_an_optional_part_may_be_left_out() {
    let found = adapted(&fixture_graph(), EGGS, &plan(vec![], &["enrichment"]));
    assert_eq!(found.unwrap_err(), AdaptError::RequiredLeftOut);
    let unknown = adapted(&fixture_graph(), EGGS, &plan(vec![], &["garnish"]));
    assert_eq!(unknown.unwrap_err(), AdaptError::UnlistedFood);
}

#[test]
fn a_placeholder_for_a_part_the_recipe_lacks_is_refused() {
    let mut graph = fixture_graph();
    change_thing(&mut graph, EGGS, |recipe| recipe["steps"][0]["text"] = json!("Crack the {yolk} in."));
    let found = adapted(&graph, EGGS, &RecipePlan::default());
    assert_eq!(found.unwrap_err(), AdaptError::Step(StepError::UnknownPlaceholder("yolk".into())));
}

#[test]
fn a_plural_the_graph_lacks_is_refused() {
    let mut graph = fixture_graph();
    change_thing(&mut graph, "ingredient/egg", |egg| egg["plurals"] = json!({}));
    let found = adapted(&graph, EGGS, &RecipePlan::default());
    assert_eq!(found.unwrap_err(), AdaptError::Step(StepError::MissingPlural("ingredient/egg".into())));
}

#[test]
fn a_step_text_in_the_old_layout_with_a_stray_bracket_is_refused() {
    let mut graph = fixture_graph();
    change_thing(&mut graph, EGGS, |recipe| recipe["steps"][0]["text"] = json!("Crack the {egg:plural in."));
    let found = adapted(&graph, EGGS, &RecipePlan::default());
    assert!(matches!(found.unwrap_err(), AdaptError::Step(StepError::Malformed(_))));
}

#[test]
fn time_temperature_and_safety_points_stay_with_their_step() {
    let made = adapted(&fixture_graph(), EGGS, &RecipePlan::default()).unwrap();
    let step = &made.steps[1];
    assert_eq!(serde_json::to_value(&step.time).unwrap(), json!({"from": 4, "to": 6, "unit": "minute"}));
    assert_eq!(made.steps[2].safety_points, vec!["raw-or-lightly-cooked-egg"]);
    assert_eq!(serde_json::to_value(&made.yields).unwrap(), json!({"value": 2, "unit": "serving"}));
}
