//! Port of TomiCore's `ProgressLevelTests`: levels of one skill, counting cooks, completing a level,
//! and which level the cook is on.

use super::test_log::{catalog, complete_per_level, cooks, cooks_per_level, open_per_level, skill, TestLog};
use crate::progress::game::COOKS_TO_COMPLETE_LEVEL;
use crate::progress::SkillStatus;

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
