//! Port of TomiCore's `ProgressLogTests`: what the rules make of an imperfect log, with abandoned
//! cooks, entries from a newer menu, copies and odd orders.

use super::test_log::{catalog, complete_per_level, cook, cooks, cooks_per_level, ids, open_per_level, skill};
use super::test_log::{stopped_before_the_end, TestLog};
use crate::progress::SkillStatus;

#[test]
fn cooks_stopped_early_do_not_count() {
    let log = TestLog::new();
    let finished = cook("knife", 1);
    let stopped = stopped_before_the_end(&cook("knife", 1));
    let knife = log.progress_of("knife", &[finished, stopped]);
    assert_eq!(cooks_per_level(&knife), [1, 0]);
    assert_eq!(complete_per_level(&knife), [false, false]);
}

#[test]
fn entries_for_a_skill_the_catalog_does_not_know_are_ignored() {
    let log = TestLog::new();
    let only_knife = cooks(1, "knife", 1);
    let entries = [cooks(2, "sous-vide", 1), only_knife.clone()].concat();
    assert_eq!(log.progress(&entries), log.progress(&only_knife));
}

#[test]
fn entries_for_a_level_the_catalog_does_not_know_are_ignored() {
    let log = TestLog::new();
    let entries = [cooks(2, "knife", 3), cooks(2, "knife", 0)].concat();
    let knife = log.progress_of("knife", &entries);
    assert_eq!(cooks_per_level(&knife), [0, 0]);
    assert_eq!(knife.status(), SkillStatus::Served);
}

/// The app's entries name a lesson the library may not know; here an entry names no dish at all,
/// only its own skill and level, which is what counts. Two separate cooks of new dishes count twice.
#[test]
fn cooks_of_lessons_the_library_does_not_know_count_at_their_skill_and_level() {
    let log = TestLog::new();
    let knife = log.progress_of("knife", &[cook("knife", 1), cook("knife", 1)]);
    assert_eq!(cooks_per_level(&knife), [2, 0]);
    assert_eq!(knife.current_level(), 2);
}

#[test]
fn the_same_entry_twice_counts_once() {
    let entry = cook("knife", 1);
    let knife = TestLog::new().progress_of("knife", &[entry.clone(), entry]);
    assert_eq!(cooks_per_level(&knife), [1, 0]);
}

#[test]
fn merging_two_overlapping_logs_equals_one_log() {
    let log = TestLog::new();
    let shared = cooks(1, "knife", 1);
    let only_on_phone = [cooks(1, "knife", 1), cooks(1, "eggs", 1)].concat();
    let only_on_tablet = cooks(2, "roux", 1);
    let phone = [shared.clone(), only_on_phone.clone()].concat();
    let tablet = [shared.clone(), only_on_tablet.clone()].concat();
    let one_log = [shared, only_on_phone, only_on_tablet].concat();
    assert_eq!(log.progress(&[phone.clone(), tablet.clone()].concat()), log.progress(&one_log));
    assert_eq!(log.progress(&[tablet.clone(), phone.clone()].concat()), log.progress(&one_log));
    assert_eq!(cooks_per_level(&log.progress_of("knife", &[phone, tablet].concat())), [2, 0]);
}

#[test]
fn the_order_of_entries_does_not_matter() {
    let log = TestLog::new();
    let entries = log.cooks_completing_everything();
    let reversed: Vec<_> = entries.iter().rev().cloned().collect();
    assert_eq!(log.progress(&reversed), log.progress(&entries));
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
