use super::*;
use crate::library::tests::shipped_files;
use crate::library::Library;
use crate::special::links::tests::fixture_graph;
use crate::special::progress::OpenLevels;

const OMELETTE: &str = "spinach-hard-cheese-omelette";
const PAN_SAUCE: &str = "seared-chicken-lemon-butter-pan-sauce";

fn library() -> Library {
    Library::parse(&shipped_files()).unwrap().0
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
fn a_lesson_at_a_locked_level_is_never_offered() {
    let library = library();
    let open = OpenLevels::from_levels([(id("eggs"), 1)]);
    let tonight = Tonight { profile: &Profile::default(), available: BTreeSet::new(), open: &open };
    assert!(candidates(&library, &fixture_graph(), &tonight).is_empty(), "the omelette is eggs level 3");
}

fn lesson_ids(found: &[Candidate]) -> Vec<String> {
    found.iter().map(|candidate| candidate.lesson_id.to_string()).collect()
}

#[test]
fn a_cook_without_restrictions_is_offered_every_lesson() {
    let found = offered(&Profile::default(), &["egg"]);
    assert_eq!(lesson_ids(&found), vec![PAN_SAUCE, OMELETTE]);
}

#[test]
fn a_vegetarian_is_never_offered_chicken() {
    let profile = Profile { diet: Some(id("vegetarian")), ..Profile::default() };
    assert_eq!(lesson_ids(&offered(&profile, &[])), vec![OMELETTE]);
}

#[test]
fn a_swap_the_graph_allows_makes_a_lesson_safe() {
    let profile = Profile { avoid: ids(&["alcohol"]), ..Profile::default() };
    let pan_sauce = offered(&profile, &[]).remove(0);
    assert_eq!(pan_sauce.lesson_id, id(PAN_SAUCE));
    let swap = AppliedSwap { removed: id("white-wine"), added: id("chicken-stock") };
    assert_eq!(pan_sauce.swaps, vec![swap]);
    assert!(!pan_sauce.ingredients.contains(&id("white-wine")));
}

#[test]
fn a_lesson_with_no_safe_swap_is_left_out() {
    // The omelette's cheese is milk and its only swap, parmesan, is milk too. The pan sauce's butter has no swap.
    let profile = Profile { avoid: ids(&["dairy"]), ..Profile::default() };
    assert!(offered(&profile, &[]).is_empty());
}

#[test]
fn a_lesson_the_graph_cannot_match_to_one_recipe_is_never_offered() {
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
fn a_swap_at_hand_is_used_for_a_missing_ingredient() {
    let omelette = offered(&Profile::default(), &["olive-oil", "egg"]).remove(1);
    let swap = AppliedSwap { removed: id("butter"), added: id("olive-oil") };
    assert_eq!(omelette.swaps, vec![swap]);
    assert!(!omelette.missing.contains(&id("olive-oil")));
}

#[test]
fn an_unknown_restriction_offers_nothing() {
    let profile = Profile { avoid: ids(&["mango"]), ..Profile::default() };
    assert!(offered(&profile, &[]).is_empty());
}

#[test]
fn cuisine_never_and_missing_equipment_leave_lessons_out() {
    let never_french = Profile { cuisines_never: ids(&["french"]), ..Profile::default() };
    assert!(offered(&never_french, &[]).is_empty());
    let no_stove = Profile { equipment: Some(ids(&["oven"])), ..Profile::default() };
    assert!(offered(&no_stove, &[]).is_empty());
}

#[test]
fn overlap_order_puts_fewest_missing_first() {
    let at_home = ["egg", "spinach", "vegetarian-hard-cheese", "butter", "salt", "black-pepper"];
    let order = overlap_order(offered(&Profile::default(), &at_home));
    assert_eq!(lesson_ids(&order), vec![OMELETTE, PAN_SAUCE]);
}

#[test]
fn a_model_may_only_reorder_the_list() {
    let by_overlap = overlap_order(offered(&Profile::default(), &[]));
    let model = [
        Ranked { lesson_id: id("made-up-lesson"), reason: Some("invented".into()) },
        Ranked { lesson_id: id(OMELETTE), reason: Some("quick tonight".into()) },
        Ranked { lesson_id: id(OMELETTE), reason: None },
    ];
    let order = ranked(by_overlap, &model);
    let order_ids: Vec<String> = order.iter().map(|(candidate, _)| candidate.lesson_id.to_string()).collect();
    assert_eq!(order_ids, vec![OMELETTE, PAN_SAUCE]);
    assert_eq!(order[0].1.as_deref(), Some("quick tonight"));
}
