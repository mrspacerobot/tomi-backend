use super::*;
use crate::graph::test_helpers::fixture_graph;
use crate::library::tests::shipped_files;
use crate::library::Library;
use crate::suggestions::progress::OpenLevels;

const OMELETTE: &str = "recipe/spinach-and-cheese-omelette";
const PAN_SAUCE: &str = "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce";

fn library() -> Library {
    Library::parse(&shipped_files()).unwrap()
}

fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

fn ids(raw: &[&str]) -> BTreeSet<SavedId> {
    raw.iter().map(|item| id(item)).collect()
}

/// Every level of every skill open, so tests about food are not about progress.
fn all_open(library: &Library) -> OpenLevels {
    let levels = library
        .catalog
        .skills
        .iter()
        .flat_map(|skill| skill.levels.iter().map(|level| (skill.id.clone(), level.number)));
    OpenLevels::from_levels(levels)
}

fn offered(profile: &Profile, available: &[&str]) -> Vec<Candidate> {
    let library = library();
    let open = all_open(&library);
    let tonight = Tonight { profile, available: ids(available), open: &open };
    candidates(&library, &fixture_graph(), &tonight)
}

#[test]
fn a_recipe_at_a_locked_level_is_never_offered() {
    let library = library();
    let open = OpenLevels::from_levels([(id("eggs"), 1)]);
    let tonight = Tonight { profile: &Profile::default(), available: BTreeSet::new(), open: &open };
    assert_eq!(candidate_ids(&candidates(&library, &fixture_graph(), &tonight)), vec!["recipe/soft-scrambled-eggs"]);
}

fn candidate_ids(found: &[Candidate]) -> Vec<String> {
    found.iter().map(|candidate| candidate.recipe_id.clone()).collect()
}

#[test]
fn a_cook_without_restrictions_is_offered_graph_recipes() {
    let found = offered(&Profile::default(), &["egg"]);
    assert_eq!(candidate_ids(&found), vec![PAN_SAUCE, "recipe/soft-scrambled-eggs", OMELETTE]);
}

#[test]
fn a_vegetarian_is_never_offered_chicken() {
    let profile = Profile { diet: Some(id("vegetarian")), ..Profile::default() };
    assert_eq!(candidate_ids(&offered(&profile, &[])), vec!["recipe/soft-scrambled-eggs", OMELETTE]);
}

#[test]
fn a_graph_allowed_swap_is_applied_to_a_safe_recipe_plan() {
    let profile = Profile { avoid: ids(&["alcohol"]), ..Profile::default() };
    let pan_sauce = offered(&profile, &[]).remove(0);
    assert_eq!(pan_sauce.recipe_id, PAN_SAUCE);
    let swap = AppliedSwap {
        removed: "ingredient/white-wine".into(),
        added: "ingredient/chicken-stock".into(),
        ratio: Some(crate::graph::serving::Ratio { by: "volume".into(), original: 1.0, stand_in: 1.0 }),
        caveat: Some(
            "Stock has none of the sharpness of wine. A squeeze of lemon at the end brings some of it back.".into(),
        ),
    };
    assert_eq!(pan_sauce.swaps, vec![swap]);
    assert!(!pan_sauce.ingredients.contains(&"ingredient/white-wine".to_string()));
}

#[test]
fn a_recipe_with_no_safe_swap_is_left_out() {
    // The omelette's cheese is milk and its only swap, parmesan, is milk too. The pan sauce's butter has no swap.
    let profile = Profile { avoid: ids(&["dairy"]), ..Profile::default() };
    assert!(offered(&profile, &[]).is_empty());
}

#[test]
fn a_graph_recipe_missing_from_the_serving_section_is_never_offered() {
    let library = library();
    let open = all_open(&library);
    let mut graph = fixture_graph();
    graph.serving.recipes.clear();
    let tonight = Tonight { profile: &Profile::default(), available: BTreeSet::new(), open: &open };
    assert!(candidates(&library, &graph, &tonight).is_empty());
}

#[test]
fn a_fact_a_restriction_names_that_the_graph_lacks_offers_nothing() {
    let library = library();
    let open = all_open(&library);
    let mut graph = fixture_graph();
    graph.serving.facts.remove("fact/milk");
    let profile = Profile { avoid: ids(&["dairy"]), ..Profile::default() };
    let tonight = Tonight { profile: &profile, available: BTreeSet::new(), open: &open };
    assert!(candidates(&library, &graph, &tonight).is_empty());
}

#[test]
fn a_swap_at_hand_is_used_for_a_missing_graph_ingredient() {
    let pan_sauce = offered(&Profile::default(), &["olive-oil", "egg"])
        .into_iter()
        .find(|candidate| candidate.recipe_id == PAN_SAUCE)
        .unwrap();
    let swap = AppliedSwap {
        removed: "ingredient/sunflower-oil".into(),
        added: "ingredient/olive-oil".into(),
        ratio: Some(crate::graph::serving::Ratio { by: "volume".into(), original: 1.0, stand_in: 1.0 }),
        caveat: Some("Its fruity flavour changes the finish. Keep it below the point where it smokes.".into()),
    };
    assert_eq!(pan_sauce.swaps, vec![swap]);
    assert!(!pan_sauce.missing.contains(&"ingredient/olive-oil".to_string()));
}

#[test]
fn an_unknown_restriction_offers_nothing() {
    let profile = Profile { avoid: ids(&["mango"]), ..Profile::default() };
    assert!(offered(&profile, &[]).is_empty());
}

#[test]
fn cuisine_preferences_filter_and_equipment_requirements_are_returned() {
    let never_french = Profile { cuisines_never: ids(&["french"]), ..Profile::default() };
    assert!(offered(&never_french, &[]).is_empty());
    let recipes = offered(&Profile::default(), &[]);
    assert!(recipes.iter().all(|candidate| !candidate.equipment.is_empty()));
}

#[test]
fn an_explicitly_missing_tracked_appliance_filters_the_recipe() {
    let no_stovetop = Profile { equipment: Some(BTreeSet::new()), ..Profile::default() };
    assert!(offered(&no_stovetop, &[]).is_empty());
    let stovetop = Profile { equipment: Some(ids(&["stovetop"])), ..Profile::default() };
    assert!(!offered(&stovetop, &[]).is_empty());
}

#[test]
fn overlap_order_puts_fewest_missing_first() {
    let at_home = ["egg", "spinach", "vegetarian-hard-cheese", "butter", "salt", "black-pepper"];
    let order = overlap_order(offered(&Profile::default(), &at_home));
    assert_eq!(candidate_ids(&order), vec![OMELETTE, "recipe/soft-scrambled-eggs", PAN_SAUCE]);
}

#[test]
fn a_model_may_only_reorder_the_list() {
    let by_overlap = overlap_order(offered(&Profile::default(), &[]));
    let model = [
        Ranked { recipe_id: "recipe/made-up".into(), reason: Some("invented".into()) },
        Ranked { recipe_id: OMELETTE.into(), reason: Some("quick tonight".into()) },
        Ranked { recipe_id: OMELETTE.into(), reason: None },
    ];
    let order = ranked(by_overlap, &model);
    let order_ids: Vec<String> = order.iter().map(|(candidate, _)| candidate.recipe_id.clone()).collect();
    assert_eq!(order_ids, vec![OMELETTE, "recipe/soft-scrambled-eggs", PAN_SAUCE]);
    assert_eq!(order[0].1.as_deref(), Some("quick tonight"));
}

#[test]
fn a_recipe_that_cannot_be_adapted_cleanly_is_not_offered() {
    let library = library();
    let open = all_open(&library);
    let mut graph = fixture_graph();
    crate::graph::test_helpers::change_thing(&mut graph, "recipe/soft-scrambled-eggs", |recipe| {
        recipe["steps"][0]["text"] = serde_json::json!("Crack the {yolk:plural} into a bowl.");
    });
    let tonight = Tonight { profile: &Profile::default(), available: BTreeSet::new(), open: &open };
    let found = candidate_ids(&candidates(&library, &graph, &tonight));
    assert_eq!(found, vec![PAN_SAUCE, OMELETTE]);
}

#[test]
fn an_offered_candidate_carries_the_recipe_written_for_the_cook() {
    let profile = Profile { avoid: ids(&["alcohol"]), ..Profile::default() };
    let pan_sauce = offered(&profile, &[]).remove(0);
    let lines: Vec<&str> = pan_sauce.adapted.lines.iter().map(|line| line.food.as_str()).collect();
    assert!(lines.contains(&"ingredient/chicken-stock"));
    assert!(!lines.contains(&"ingredient/white-wine"));
    assert!(pan_sauce.adapted.steps.iter().any(|step| step.text.starts_with("Pour in the chicken stock")));
}
