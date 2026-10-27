//! Ports of TomiCore's `ProgressFocusTests` (the focus is the skill Tomi cooks next: the cook's
//! choice while it can be cooked, otherwise the next in the catalog) and `ProgressMasteredCountTests`.

use crate::profile::Experience;
use crate::progress::fixtures::{catalog, cooks, id, skill, TestLog};
use crate::progress::CookEntry;

fn focus(log: &TestLog, entries: &[CookEntry], chosen: Option<&str>) -> Option<String> {
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
fn without_a_choice_the_focus_is_the_first_served_skill_in_the_catalog() {
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
fn the_fallback_skips_locked_and_mastered_skills_in_catalog_order() {
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

#[test]
fn nothing_is_mastered_before_the_first_cook() {
    assert_eq!(TestLog::new().progress(&[]).mastered_count(), 0);
}

#[test]
fn a_skill_counts_once_every_level_is_complete() {
    let log = TestLog::new();
    let first_level = cooks(2, "knife", 1);
    assert_eq!(log.progress(&first_level).mastered_count(), 0);
    let both_levels = [first_level, cooks(2, "knife", 2)].concat();
    assert_eq!(log.progress(&both_levels).mastered_count(), 1);
}

#[test]
fn skills_granted_by_experience_count_with_the_cooked_ones() {
    let log = TestLog::with_experience(Experience::Comfortable);
    assert_eq!(log.progress(&[]).mastered_count(), 2);
    let roux = [cooks(2, "roux", 1), cooks(2, "roux", 2)].concat();
    assert_eq!(log.progress(&roux).mastered_count(), 3);
}
