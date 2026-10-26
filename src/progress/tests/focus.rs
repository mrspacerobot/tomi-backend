//! Port of TomiCore's `ProgressFocusTests`: the focus is the skill Tomi cooks next, the cook's choice
//! while it can be cooked, otherwise the next on the menu.

use super::test_log::{catalog, cooks, id, skill, TestLog};

fn focus(log: &TestLog, entries: &[crate::progress::CookEntry], chosen: Option<&str>) -> Option<String> {
    let chosen = chosen.map(id);
    let focus = log.progress(entries).focus(chosen.as_ref());
    focus.map(|skill| skill.to_string())
}

fn some(raw: &str) -> Option<String> {
    Some(raw.to_string())
}

#[test]
fn a_served_unmastered_choice_is_kept() {
    assert_eq!(focus(&TestLog::new(), &[], Some("eggs")), some("eggs"));
}

#[test]
fn without_a_choice_the_focus_is_the_first_served_skill_on_the_menu() {
    assert_eq!(focus(&TestLog::new(), &[], None), some("knife"));
}

#[test]
fn a_locked_choice_falls_back_to_the_first_served_skill() {
    assert_eq!(focus(&TestLog::new(), &[], Some("roux")), some("knife"));
}

#[test]
fn a_mastered_choice_falls_back_to_the_first_served_unmastered_skill() {
    let log = TestLog::new();
    let entries = [cooks(2, "knife", 1), cooks(2, "knife", 2)].concat();
    assert_eq!(focus(&log, &entries, Some("knife")), some("eggs"));
}

#[test]
fn a_choice_the_catalog_does_not_know_falls_back() {
    assert_eq!(focus(&TestLog::new(), &[], Some("sous-vide")), some("knife"));
}

#[test]
fn the_fallback_skips_locked_and_mastered_skills_in_menu_order() {
    let skills = [skill("knife", "basics"), skill("eggs", "basics").needs(&["roux"]), skill("roux", "sauces")];
    let log = TestLog::with_catalog(catalog(&["basics", "sauces"], &skills));
    let knife_mastered = [cooks(2, "knife", 1), cooks(2, "knife", 2)].concat();
    assert_eq!(focus(&log, &knife_mastered, Some("eggs")), some("roux"));
}

#[test]
fn there_is_no_focus_when_everything_is_mastered() {
    let log = TestLog::new();
    assert_eq!(focus(&log, &log.cooks_completing_everything(), Some("knife")), None);
}
