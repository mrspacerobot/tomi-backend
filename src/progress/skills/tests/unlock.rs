//! Ports of TomiCore's `ProgressUnlockTests` (which skills are locked, served or mastered) and
//! `ProgressExperienceTests` ("comfortable" hands the cook the first course as mastered; the fixture
//! catalog has knife and eggs in the first course and roux, which needs knife, in the second).

use crate::profile::Experience;
use crate::progress::fixtures::{
    catalog, complete_per_level, cooks, cooks_per_level, fixture_skills, granted_per_level, id, ids,
};
use crate::progress::fixtures::{open_per_level, skill, TestLog};
use crate::progress::{granted_skills, CookEntry, SkillStatus};
use std::collections::BTreeSet;

fn locked(missing: &[&str]) -> SkillStatus {
    SkillStatus::Locked { missing: ids(missing) }
}

#[test]
fn an_empty_log_serves_only_skills_without_prerequisites() {
    let progress = TestLog::new().progress(&[]);
    let skills: Vec<String> = progress.skills.iter().map(|skill| skill.skill.to_string()).collect();
    assert_eq!(skills, ["knife", "eggs", "roux"]);
    let statuses: Vec<SkillStatus> = progress.skills.iter().map(|skill| skill.status()).collect();
    assert_eq!(statuses, [SkillStatus::Served, SkillStatus::Served, locked(&["knife"])]);
    assert!(progress.skills.iter().all(|skill| cooks_per_level(skill) == [0, 0]));
}

#[test]
fn a_skill_is_served_once_level_one_of_its_prerequisite_is_complete() {
    let log = TestLog::new();
    let roux = log.progress_of("roux", &cooks(2, "knife", 1));
    assert_eq!(roux.status(), SkillStatus::Served);
    assert!(roux.missing_prerequisites.is_empty());
    assert_eq!(open_per_level(&roux), [true, false]);
}

#[test]
fn one_cook_of_a_prerequisite_is_not_enough() {
    let log = TestLog::new();
    assert_eq!(log.progress_of("roux", &cooks(1, "knife", 1)).status(), locked(&["knife"]));
}

#[test]
fn a_later_level_of_a_prerequisite_does_not_stand_in_for_level_one() {
    let log = TestLog::new();
    assert_eq!(log.progress_of("roux", &cooks(2, "knife", 2)).status(), locked(&["knife"]));
}

#[test]
fn a_locked_skill_names_only_the_prerequisites_still_missing() {
    let roux = skill("roux", "sauces").needs(&["knife", "eggs"]);
    let log = TestLog::with_catalog(catalog(
        &["basics", "sauces"],
        &[skill("knife", "basics"), skill("eggs", "basics"), roux],
    ));
    assert_eq!(log.progress_of("roux", &[]).status(), locked(&["knife", "eggs"]));
    assert_eq!(log.progress_of("roux", &cooks(2, "knife", 1)).status(), locked(&["eggs"]));
    let both = [cooks(2, "knife", 1), cooks(2, "eggs", 1)].concat();
    assert_eq!(log.progress_of("roux", &both).status(), SkillStatus::Served);
}

#[test]
fn a_skill_is_mastered_when_all_its_levels_are_complete() {
    let log = TestLog::new();
    let level_one = cooks(2, "knife", 1);
    assert_eq!(log.progress_of("knife", &level_one).status(), SkillStatus::Served);
    let knife = log.progress_of("knife", &[level_one, cooks(2, "knife", 2)].concat());
    assert_eq!(knife.status(), SkillStatus::Mastered);
    assert!(knife.is_mastered());
    assert!(knife.is_served());
}

#[test]
fn the_whole_catalog_is_mastered_when_every_level_of_every_skill_is_complete() {
    let log = TestLog::new();
    let progress = log.progress(&log.cooks_completing_everything());
    let statuses: Vec<SkillStatus> = progress.skills.iter().map(|skill| skill.status()).collect();
    assert_eq!(statuses, vec![SkillStatus::Mastered; 3]);
    assert!(progress.skills.iter().all(|skill| complete_per_level(skill).iter().all(|complete| *complete)));
}

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
fn a_granted_catalog_is_fully_mastered_once_the_rest_is_cooked() {
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
fn a_catalog_without_courses_grants_nothing() {
    let empty = catalog(&[], &[]);
    assert!(granted_skills(&empty, Experience::Comfortable).is_empty());
    let log = TestLog { catalog: empty, experience: Experience::Comfortable };
    assert!(log.progress(&[]).skills.is_empty());
}
