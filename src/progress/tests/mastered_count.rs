//! Port of TomiCore's `ProgressMasteredCountTests`: how many techniques of the menu are mastered.

use super::test_log::{cooks, TestLog};
use crate::profile::Experience;

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
