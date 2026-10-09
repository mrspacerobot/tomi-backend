use super::*;
use crate::domain::library::tests::shipped_files;
use crate::domain::library::Library;

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

fn offered(profile: &Profile, available: &[&str]) -> Vec<Candidate> {
    let tonight = Tonight { profile, available: ids(available) };
    candidates(&library(), &tonight)
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
fn a_listed_swap_makes_a_lesson_safe() {
    let profile = Profile { avoid: ids(&["alcohol"]), ..Profile::default() };
    let pan_sauce = offered(&profile, &[]).remove(0);
    assert_eq!(pan_sauce.lesson_id, id(PAN_SAUCE));
    let swap = AppliedSwap { replace: id("white-wine"), with: id("chicken-stock") };
    assert_eq!(pan_sauce.swaps, vec![swap]);
    assert!(!pan_sauce.ingredients.contains(&id("white-wine")));
}

#[test]
fn a_lesson_with_no_safe_swap_is_left_out() {
    // Cheese is dairy and has no listed swap, so the omelette cannot be made dairy-free.
    let profile = Profile { avoid: ids(&["dairy"]), ..Profile::default() };
    assert!(!lesson_ids(&offered(&profile, &[])).contains(&OMELETTE.to_string()));
}

#[test]
fn a_swap_at_hand_is_used_for_a_missing_ingredient() {
    let omelette = offered(&Profile::default(), &["olive-oil", "egg"]).remove(1);
    let swap = AppliedSwap { replace: id("butter"), with: id("olive-oil") };
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
