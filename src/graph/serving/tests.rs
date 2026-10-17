use super::*;
use serde_json::json;

fn line(thing: &str, part: &str, unsafe_for: &[&str]) -> serde_json::Value {
    json!({ "thing": thing, "part": part, "optional": false, "unsafe_for": unsafe_for, "swaps": [] })
}

fn optional(mut line: serde_json::Value) -> serde_json::Value {
    line["optional"] = json!(true);
    line
}

fn with_swap(mut line: serde_json::Value, swap: serde_json::Value) -> serde_json::Value {
    line["swaps"] = json!([swap]);
    line
}

fn oil() -> serde_json::Value {
    json!({
        "thing": "ingredient/olive-oil", "unsafe_for": [], "basis": "stand-in",
        "ratio": { "stand_in": 1, "original": 1, "by": "weight" }, "caveat": "No butter aroma.", "tier": "drafted"
    })
}

fn parmesan() -> serde_json::Value {
    json!({
        "thing": "ingredient/parmesan", "unsafe_for": ["fact/milk", "fact/slaughter-product"],
        "basis": "stand-in", "ratio": null, "caveat": null, "tier": "drafted"
    })
}

fn omelette_lines() -> Vec<serde_json::Value> {
    vec![
        line("ingredient/egg", "egg", &["fact/egg"]),
        with_swap(line("ingredient/butter", "fat", &["fact/milk"]), oil()),
        with_swap(line("ingredient/cheese", "cheese", &["fact/milk"]), parmesan()),
        optional(line("ingredient/chives", "garnish", &["fact/milk"])),
    ]
}

fn section() -> Serving {
    let recipe = json!({
        "makes": "dish/omelette", "tier": "drafted", "techniques": ["technique/roll-an-omelette"],
        "ingredients": [], "equipment": [], "unsafe_for": [], "optional_parts": [], "lines": omelette_lines()
    });
    let facts = ["fact/egg", "fact/milk", "fact/slaughter-product"];
    serde_json::from_value(json!({ "facts": facts, "recipes": { "recipe/omelette": recipe } })).unwrap()
}

fn avoiding(facts: &[&str]) -> BTreeSet<String> {
    facts.iter().map(|fact| fact.to_string()).collect()
}

fn asks(facts: &[&str]) -> CookFacts {
    CookFacts { avoided: avoiding(facts), at_hand: BTreeSet::new() }
}

fn plan(facts: &[&str]) -> Result<Option<RecipePlan>, ServingError> {
    section().plan("recipe/omelette", &asks(facts))
}

#[test]
fn a_cook_who_avoids_nothing_gets_the_recipe_as_written() {
    let plan = plan(&[]).unwrap().unwrap();
    assert!(plan.swaps.is_empty());
    assert!(plan.left_out.is_empty());
}

#[test]
fn a_recipe_with_a_line_that_has_no_safe_way_is_refused() {
    assert_eq!(plan(&["fact/egg"]).unwrap(), None);
}

#[test]
fn a_line_whose_only_swap_is_also_unsafe_refuses_the_recipe() {
    assert_eq!(plan(&["fact/milk"]).unwrap(), None);
}

#[test]
fn a_safe_swap_is_used_and_carries_its_ratio_and_caveat() {
    let mut milk_free = section();
    milk_free.recipes.get_mut("recipe/omelette").unwrap().lines.remove(2);
    let plan = milk_free.plan("recipe/omelette", &asks(&["fact/milk"])).unwrap().unwrap();
    assert_eq!(plan.swaps.len(), 1);
    assert_eq!(plan.swaps[0].original, "ingredient/butter");
    assert_eq!(plan.swaps[0].instead, "ingredient/olive-oil");
    assert_eq!(plan.swaps[0].caveat.as_deref(), Some("No butter aroma."));
}

#[test]
fn an_optional_line_with_no_way_is_left_out_and_does_not_refuse_the_recipe() {
    let mut garnish_only = section();
    garnish_only.recipes.get_mut("recipe/omelette").unwrap().lines.retain(|line| line.part == "garnish");
    let plan = garnish_only.plan("recipe/omelette", &asks(&["fact/milk"])).unwrap().unwrap();
    assert_eq!(plan.left_out, vec!["garnish".to_string()]);
}

#[test]
fn a_fact_the_graph_does_not_know_is_an_error_never_a_pass() {
    let error = plan(&["fact/unheard-of"]).unwrap_err();
    assert_eq!(error, ServingError::UnknownFact("fact/unheard-of".to_string()));
}

#[test]
fn a_recipe_the_graph_does_not_know_is_an_error() {
    let error = section().plan("recipe/nope", &asks(&[])).unwrap_err();
    assert_eq!(error, ServingError::UnknownRecipe("recipe/nope".to_string()));
}

#[test]
fn a_section_with_a_field_this_build_does_not_know_is_refused() {
    let text = json!({ "facts": [], "recipes": {}, "surprise": 1 }).to_string();
    assert!(serde_json::from_str::<Serving>(&text).is_err());
}

fn at_hand(foods: &[&str]) -> CookFacts {
    CookFacts { avoided: BTreeSet::new(), at_hand: avoiding(foods) }
}

#[test]
fn a_swap_the_cook_has_replaces_a_safe_original_they_do_not_have() {
    let cook = at_hand(&["ingredient/olive-oil", "ingredient/egg", "ingredient/cheese"]);
    let plan = section().plan("recipe/omelette", &cook).unwrap().unwrap();
    assert_eq!(plan.swaps.len(), 1);
    assert_eq!(plan.swaps[0].instead, "ingredient/olive-oil");
}

#[test]
fn the_original_is_kept_when_the_cook_has_it_even_if_a_swap_is_also_at_hand() {
    let cook = at_hand(&["ingredient/olive-oil", "ingredient/butter"]);
    assert!(section().plan("recipe/omelette", &cook).unwrap().unwrap().swaps.is_empty());
}

#[test]
fn a_safe_swap_at_hand_is_preferred_to_the_first_safe_swap() {
    let mut recipe = section();
    let line = &mut recipe.recipes.get_mut("recipe/omelette").unwrap().lines[1];
    let mut second = line.swaps[0].clone();
    second.thing = "ingredient/ghee-free-spread".into();
    line.swaps.push(second);
    let cook = CookFacts { avoided: avoiding(&["fact/milk"]), at_hand: avoiding(&["ingredient/ghee-free-spread"]) };
    recipe.recipes.get_mut("recipe/omelette").unwrap().lines.remove(2);
    let plan = recipe.plan("recipe/omelette", &cook).unwrap().unwrap();
    assert_eq!(plan.swaps[0].instead, "ingredient/ghee-free-spread");
}
