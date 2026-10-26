//! Port of TomiCore's `ProgressExperienceTests`: "comfortable" hands the cook the first course as
//! mastered. The fixture menu has knife and eggs in the first course and roux, which needs knife,
//! in the second.

use super::test_log::{catalog, complete_per_level, cooks, cooks_per_level, fixture_skills, granted_per_level};
use super::test_log::{id, ids, open_per_level, TestLog};
use crate::profile::Experience;
use crate::progress::{granted_skills, CookEntry, SkillStatus};
use std::collections::BTreeSet;

fn comfortable() -> TestLog {
    TestLog::with_experience(Experience::Comfortable)
}

fn statuses(log: &TestLog, entries: &[CookEntry]) -> Vec<SkillStatus> {
    log.progress(entries).skills.iter().map(|skill| skill.status()).collect()
}

#[test]
fn comfortable_counts_every_technique_of_the_first_course_as_mastered() {
    let progress = comfortable().progress(&[]);
    assert_eq!(progress.skill(&id("knife")).unwrap().status(), SkillStatus::Mastered);
    assert_eq!(progress.skill(&id("eggs")).unwrap().status(), SkillStatus::Mastered);
    assert_eq!(complete_per_level(&comfortable().progress_of("knife", &[])), [true, true]);
    assert_eq!(comfortable().progress_of("knife", &[]).current_level(), 2);
}

#[test]
fn comfortable_unlocks_the_second_course() {
    let roux = comfortable().progress_of("roux", &[]);
    assert_eq!(roux.status(), SkillStatus::Served);
    assert!(roux.missing_prerequisites.is_empty());
    assert_eq!(open_per_level(&roux), [true, false]);
    assert_eq!(complete_per_level(&roux), [false, false]);
    assert_eq!(comfortable().progress(&[]).focus(None), Some(id("roux")));
}

#[test]
fn granted_levels_count_no_cooks_that_were_never_cooked() {
    let knife = comfortable().progress_of("knife", &[]);
    assert_eq!(cooks_per_level(&knife), [0, 0]);
    assert_eq!(granted_per_level(&knife), [true, true]);
    assert_eq!(granted_per_level(&comfortable().progress_of("roux", &[])), [false, false]);
}

#[test]
fn the_other_answers_grant_nothing() {
    for experience in [Experience::Never, Experience::Basics] {
        let log = TestLog::with_experience(experience);
        let locked = SkillStatus::Locked { missing: ids(&["knife"]) };
        assert_eq!(statuses(&log, &[]), [SkillStatus::Served, SkillStatus::Served, locked]);
        let progress = log.progress(&[]);
        assert!(progress.skills.iter().all(|skill| granted_per_level(skill) == [false, false]));
        assert!(granted_skills(&log.catalog, experience).is_empty());
    }
}

#[test]
fn real_cooks_count_on_top_of_granted_skills() {
    let log = comfortable();
    let entries = [cooks(1, "knife", 1), cooks(2, "roux", 1)].concat();
    let progress = log.progress(&entries);
    assert_eq!(cooks_per_level(progress.skill(&id("knife")).unwrap()), [1, 0]);
    assert_eq!(progress.skill(&id("knife")).unwrap().status(), SkillStatus::Mastered);
    assert_eq!(complete_per_level(progress.skill(&id("roux")).unwrap()), [true, false]);
    assert_eq!(progress.skill(&id("roux")).unwrap().current_level(), 2);
}

#[test]
fn a_granted_menu_is_fully_mastered_once_the_rest_is_cooked() {
    let log = comfortable();
    let entries = [cooks(2, "roux", 1), cooks(2, "roux", 2)].concat();
    assert_eq!(statuses(&log, &entries), vec![SkillStatus::Mastered; 3]);
    assert_eq!(log.progress(&entries).focus(None), None);
}

#[test]
fn the_first_course_is_read_from_the_catalog() {
    let sauces_first = catalog(&["sauces", "basics"], &fixture_skills());
    assert_eq!(granted_skills(&sauces_first, Experience::Comfortable), BTreeSet::from([id("roux")]));
    let log = TestLog { catalog: sauces_first, experience: Experience::Comfortable };
    assert_eq!(statuses(&log, &[]), [SkillStatus::Served, SkillStatus::Served, SkillStatus::Mastered]);
}

#[test]
fn a_menu_without_courses_grants_nothing() {
    let empty = catalog(&[], &[]);
    assert!(granted_skills(&empty, Experience::Comfortable).is_empty());
    let log = TestLog { catalog: empty, experience: Experience::Comfortable };
    assert!(log.progress(&[]).skills.is_empty());
}
