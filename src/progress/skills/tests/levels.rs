//! Ports of TomiCore's `ProgressLevelTests` and the level cases of `ProgressLogTests`, plus the
//! levels suggestions may offer on the shipped catalog.

use crate::library::tests::shipped_files;
use crate::library::{Catalog, Library};
use crate::profile::Experience;
use crate::progress::fixtures::{
    catalog, complete_per_level, cooks, cooks_per_level, id, ids, open_per_level, skill, TestLog,
};
use crate::progress::game::COOKS_TO_COMPLETE_LEVEL;
use crate::progress::{open_levels, CookCounts, OpenLevels, SkillStatus};

#[test]
fn a_level_takes_two_finished_cooks() {
    assert_eq!(COOKS_TO_COMPLETE_LEVEL, 2);
}

#[test]
fn cooks_are_counted_per_level() {
    let log = TestLog::new();
    let entries = [cooks(2, "knife", 1), cooks(1, "knife", 2)].concat();
    let knife = log.progress_of("knife", &entries);
    let numbers: Vec<u8> = knife.levels.iter().map(|level| level.number).collect();
    assert_eq!(numbers, [1, 2]);
    assert_eq!(cooks_per_level(&knife), [2, 1]);
}

#[test]
fn a_level_is_not_complete_one_cook_short() {
    let log = TestLog::new();
    let knife = log.progress_of("knife", &cooks(COOKS_TO_COMPLETE_LEVEL - 1, "knife", 1));
    assert_eq!(complete_per_level(&knife), [false, false]);
    assert_eq!(knife.current_level(), 1);
}

#[test]
fn completing_a_level_opens_the_next_one() {
    let log = TestLog::new();
    let knife = log.progress_of("knife", &cooks(COOKS_TO_COMPLETE_LEVEL, "knife", 1));
    assert_eq!(complete_per_level(&knife), [true, false]);
    assert_eq!(open_per_level(&knife), [true, true]);
    assert_eq!(knife.current_level(), 2);
}

#[test]
fn the_next_level_stays_closed_until_the_one_before_is_complete() {
    let log = TestLog::new();
    let knife = log.progress_of("knife", &cooks(1, "knife", 1));
    assert_eq!(open_per_level(&knife), [true, false]);
}

#[test]
fn extra_cooks_at_a_complete_level_do_not_spill_into_the_next() {
    let log = TestLog::new();
    let knife = log.progress_of("knife", &cooks(5, "knife", 1));
    assert_eq!(cooks_per_level(&knife), [5, 0]);
    assert_eq!(knife.current_level(), 2);
}

#[test]
fn no_level_of_a_locked_skill_is_open() {
    let roux = TestLog::new().progress_of("roux", &[]);
    assert_eq!(open_per_level(&roux), [false, false]);
    assert_eq!(roux.current_level(), 1);
}

#[test]
fn a_skill_with_one_level_is_mastered_by_completing_it() {
    let log = TestLog::with_catalog(catalog(&["basics"], &[skill("knife", "basics").levels(&[1])]));
    assert_eq!(log.progress_of("knife", &cooks(1, "knife", 1)).status(), SkillStatus::Served);
    let mastered = log.progress_of("knife", &cooks(2, "knife", 1));
    assert_eq!(mastered.status(), SkillStatus::Mastered);
    assert_eq!(mastered.current_level(), 1);
}

#[test]
fn a_mastered_skill_stays_on_its_last_level() {
    let log = TestLog::new();
    let entries = [cooks(2, "knife", 1), cooks(2, "knife", 2)].concat();
    assert_eq!(log.progress_of("knife", &entries).current_level(), 2);
}

/// The chosen rule: every finished cook counts at its own level, whether or not that level was open.
#[test]
fn a_cook_at_a_level_not_yet_open_still_counts_at_that_level() {
    let log = TestLog::new();
    let early = cooks(2, "knife", 2);
    let knife = log.progress_of("knife", &early);
    assert_eq!(cooks_per_level(&knife), [0, 2]);
    assert_eq!(complete_per_level(&knife), [false, true]);
    assert_eq!(open_per_level(&knife), [true, false]);
    assert_eq!(knife.current_level(), 1);
    assert_eq!(knife.status(), SkillStatus::Served);
    let caught_up = [early, cooks(2, "knife", 1)].concat();
    assert_eq!(log.progress_of("knife", &caught_up).status(), SkillStatus::Mastered);
}

#[test]
fn a_complete_level_does_not_open_the_next_while_an_earlier_one_is_incomplete() {
    let log = TestLog::with_catalog(catalog(&["basics"], &[skill("knife", "basics").levels(&[1, 2, 3])]));
    let early = cooks(2, "knife", 2);
    assert_eq!(open_per_level(&log.progress_of("knife", &early)), [true, false, false]);
    let caught_up = [early, cooks(2, "knife", 1)].concat();
    assert_eq!(open_per_level(&log.progress_of("knife", &caught_up)), [true, true, true]);
    assert_eq!(log.progress_of("knife", &caught_up).current_level(), 3);
}

/// A newer catalog may add a prerequisite to a skill the cook has already finished.
#[test]
fn a_skill_with_every_level_complete_shows_as_mastered_even_if_a_prerequisite_is_missing() {
    let log = TestLog::new();
    let early = [cooks(2, "roux", 1), cooks(2, "roux", 2)].concat();
    let roux = log.progress_of("roux", &early);
    assert_eq!(roux.status(), SkillStatus::Mastered);
    assert_eq!(roux.missing_prerequisites, ids(&["knife"]));
    assert!(!roux.is_served());
}

#[test]
fn cooks_of_a_locked_skill_are_kept_for_when_it_is_served() {
    let log = TestLog::new();
    let early = cooks(2, "roux", 1);
    let locked = log.progress_of("roux", &early);
    assert_eq!(locked.status(), SkillStatus::Locked { missing: ids(&["knife"]) });
    assert_eq!(cooks_per_level(&locked), [2, 0]);
    let served = log.progress_of("roux", &[early, cooks(2, "knife", 1)].concat());
    assert_eq!(served.status(), SkillStatus::Served);
    assert_eq!(served.current_level(), 2);
}

fn shipped_catalog() -> Catalog {
    Library::parse(&shipped_files()).unwrap().catalog
}

fn open(experience: Experience, counts: &CookCounts) -> OpenLevels {
    open_levels(&shipped_catalog(), experience, counts)
}

fn knife_cuts(count: u32) -> CookCounts {
    CookCounts::from_log(&cooks(count, "knife-cuts", 1))
}

#[test]
fn a_new_cook_starts_at_level_one_of_skills_without_prerequisites() {
    let open = open(Experience::Basics, &CookCounts::default());
    assert!(open.contains(&id("knife-cuts"), 1));
    assert!(open.contains(&id("eggs"), 1));
    assert!(!open.contains(&id("eggs"), 2));
    assert!(!open.contains(&id("saute"), 1), "saute needs knife cuts first");
}

#[test]
fn comfortable_counts_the_first_course_as_complete() {
    let open = open(Experience::Comfortable, &CookCounts::default());
    assert!(open.contains(&id("eggs"), 3));
    assert!(open.contains(&id("saute"), 1));
    assert!(!open.contains(&id("saute"), 2));
    assert!(!open.contains(&id("sear"), 1), "sear needs saute level 1 cooked twice");
}

#[test]
fn two_cooks_complete_a_level_and_open_what_follows() {
    let open = open(Experience::Never, &knife_cuts(2));
    assert!(open.contains(&id("knife-cuts"), 2));
    assert!(open.contains(&id("saute"), 1));
    assert!(open.contains(&id("stocks"), 1));
}

#[test]
fn one_cook_is_not_enough() {
    assert!(!open(Experience::Never, &knife_cuts(1)).contains(&id("knife-cuts"), 2));
}

#[test]
fn a_mastered_skill_without_its_prerequisites_opens_no_level() {
    let log = TestLog::new();
    let roux = CookCounts::from_log(&[cooks(2, "roux", 1), cooks(2, "roux", 2)].concat());
    let open = open_levels(&log.catalog, Experience::Basics, &roux);
    assert!(!open.contains(&id("roux"), 1));
    assert!(open.contains(&id("knife"), 1));
}
