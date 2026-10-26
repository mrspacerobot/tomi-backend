//! Port of TomiCore's `ProgressUnlockTests`: which skills are locked, served or mastered.

use super::test_log::{catalog, complete_per_level, cooks, cooks_per_level, ids, open_per_level, skill, TestLog};
use crate::progress::SkillStatus;

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
fn the_whole_menu_is_mastered_when_every_level_of_every_skill_is_complete() {
    let log = TestLog::new();
    let progress = log.progress(&log.cooks_completing_everything());
    let statuses: Vec<SkillStatus> = progress.skills.iter().map(|skill| skill.status()).collect();
    assert_eq!(statuses, vec![SkillStatus::Mastered; 3]);
    assert!(progress.skills.iter().all(|skill| complete_per_level(skill).iter().all(|complete| *complete)));
}
