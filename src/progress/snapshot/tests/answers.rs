//! What the snapshot holds: zeros for a new cook, the cook's answers, the rank's words and what the log counted.

use super::{utc, Request};
use crate::kernel::language::Language;
use crate::library::tests::shipped_files;
use crate::library::LibraryFiles;
use crate::profile::Experience;
use crate::progress::fixtures::{cook, cooks, id, october, stopped_before_the_end};
use crate::progress::{CookEntry, SkillStatusAnswer};
use jiff::Timestamp;
use serde_json::{json, Value};

fn tuesday_evening() -> Timestamp {
    utc(october(6), (18, 0, 0))
}

fn new_request() -> Request {
    Request::new("UTC", 2, tuesday_evening())
}

#[test]
fn a_new_cook_gets_a_complete_snapshot_with_its_versions_time_and_calendar() {
    let snapshot = new_request().json(&[]);
    let versions = (&snapshot["schemaVersion"], &snapshot["rulesVersion"], &snapshot["catalogVersion"]);
    assert_eq!(versions, (&json!(1), &json!(1), &json!(1)));
    assert_eq!(snapshot["computedAt"], "2026-10-06T18:00:00Z");
    assert_eq!(snapshot["calendar"], json!({"timeZone": "UTC", "firstWeekday": 2}));
}

#[test]
fn a_new_cook_has_no_xp_no_streak_and_nothing_counted() {
    let snapshot = new_request().json(&[]);
    assert_eq!(snapshot["xp"], 0);
    assert_eq!(snapshot["rank"], json!({"id": "novice", "name": "Novice", "tier": 1, "startsAt": 0, "nextAt": 150}));
    assert_eq!(snapshot["streak"], zero_streak());
    assert_eq!((&snapshot["masteredCount"], &snapshot["focusSkill"]), (&json!(0), &json!("knife-cuts")));
    assert_eq!(snapshot["counted"], json!({"finishedCooks": 0, "ignoredUnknownSkillOrLevel": 0}));
}

/// No streak in the Monday-first week of 5 October, on Tuesday evening in UTC.
fn zero_streak() -> Value {
    json!({
        "weeks": 0, "weeksBeforeThisWeek": 0, "daysThisWeek": 0, "goal": 3, "daysLeftInWeek": 6,
        "thisWeekMet": false, "atRisk": false, "lost": false,
        "weekStartsAt": "2026-10-05T00:00:00Z", "weekEndsAt": "2026-10-12T00:00:00Z",
    })
}

#[test]
fn a_new_cook_sees_every_skill_of_the_catalog_with_no_cooks() {
    let request = new_request();
    let snapshot = request.json(&[]);
    let skills = snapshot["skills"].as_array().unwrap();
    let ids: Vec<&str> = skills.iter().map(|skill| skill["skill"].as_str().unwrap()).collect();
    let catalog: Vec<String> = request.library.catalog.skills.iter().map(|skill| skill.id.to_string()).collect();
    assert_eq!(ids, catalog);
    assert!(skills.iter().flat_map(|skill| skill["levels"].as_array().unwrap()).all(|level| level["cooks"] == 0));
    assert_eq!((&skills[2]["status"], &skills[2]["missingPrerequisites"]), (&json!("locked"), &json!(["knife-cuts"])));
}

#[test]
fn a_new_cook_starts_a_skill_without_prerequisites_at_its_open_first_level() {
    let snapshot = new_request().json(&[]);
    let eggs = &snapshot["skills"][1];
    assert_eq!((&eggs["skill"], &eggs["status"], &eggs["currentLevel"]), (&json!("eggs"), &json!("served"), &json!(1)));
    assert_eq!(eggs["missingPrerequisites"], json!([]));
    let first_level =
        json!({"number": 1, "cooks": 0, "cooksToComplete": 2, "open": true, "granted": false, "complete": false});
    assert_eq!(eggs["levels"][0], first_level);
    assert_eq!(eggs["levels"][1]["open"], false);
}

#[test]
fn comfortable_grants_the_first_course_with_no_xp() {
    let mut request = new_request();
    request.answers.experience = Experience::Comfortable;
    let snapshot = request.snapshot(&[]);
    assert_eq!(snapshot.xp, 0);
    let first_course = ["knife-cuts", "eggs", "stocks"];
    for skill in snapshot.catalog.skills.iter().filter(|skill| first_course.contains(&skill.skill.as_str())) {
        assert_eq!(skill.status, SkillStatusAnswer::Mastered, "{}", skill.skill);
        assert!(skill.levels.iter().all(|level| level.granted && level.complete && level.cooks == 0));
    }
    assert_eq!(snapshot.catalog.mastered_count, 3);
    assert_eq!(snapshot.counted.finished_cooks, 0);
}

#[test]
fn the_weekly_goal_and_focus_skill_come_from_the_cooks_answers() {
    let mut request = new_request();
    request.answers.weekly_goal = 5;
    request.answers.focus_skill = Some(id("eggs"));
    let snapshot = request.snapshot(&[]);
    assert_eq!(snapshot.streak.goal, 5);
    assert_eq!(snapshot.catalog.focus_skill, Some(id("eggs")));
    request.answers.focus_skill = Some(id("roux"));
    assert_eq!(request.snapshot(&[]).catalog.focus_skill, Some(id("knife-cuts")), "a locked choice falls back");
}

fn ranks_in_german() -> LibraryFiles {
    let mut ranks: Value = serde_json::from_str(&shipped_files().ranks).unwrap();
    ranks["ranks"][0]["name"]["de"] = json!("Anfänger");
    LibraryFiles { ranks: ranks.to_string(), ..shipped_files() }
}

#[test]
fn the_rank_name_follows_the_language_and_the_id_does_not() {
    let mut request = new_request().with_library(&ranks_in_german());
    let english = request.snapshot(&[]).rank;
    request.language = Language::from_accept_language(Some("de-DE,de;q=0.9"));
    let german = request.snapshot(&[]).rank;
    assert_eq!((english.id, english.name.as_str()), ("novice", "Novice"));
    assert_eq!((german.id, german.name.as_str()), ("novice", "Anfänger"));
    request.language = Language::from_accept_language(Some("fr"));
    assert_eq!(request.snapshot(&[]).rank.name, "Novice", "a language without the name falls back to English");
}

#[test]
fn cooks_of_a_skill_or_level_the_catalog_does_not_know_earn_xp_and_are_counted_as_ignored() {
    let log = [cook("eggs", 1), cook("eggs", 9), cook("pastry", 1), stopped_before_the_end(&cook("eggs", 1))];
    let snapshot = new_request().snapshot(&log);
    assert_eq!(snapshot.counted.finished_cooks, 3);
    assert_eq!(snapshot.counted.ignored_unknown_skill_or_level, 2);
    assert_eq!(snapshot.xp, 180);
    assert_eq!(snapshot.catalog.skills[1].levels[0].cooks, 1);
}

#[test]
fn computed_at_is_the_request_moment_to_the_second() {
    let request =
        Request::new("UTC", 2, tuesday_evening().checked_add(jiff::SignedDuration::from_millis(1_750)).unwrap());
    assert_eq!(request.json(&[])["computedAt"], "2026-10-06T18:00:01Z");
}

#[test]
fn the_same_log_in_any_order_and_with_copies_gives_the_same_snapshot() {
    let log: Vec<CookEntry> = cooks(3, "eggs", 1).into_iter().chain(cooks(2, "knife-cuts", 1)).collect();
    let mut reversed_with_copies = log.clone();
    reversed_with_copies.reverse();
    reversed_with_copies.extend(log.iter().cloned());
    let request = new_request();
    assert_eq!(request.snapshot(&log), request.snapshot(&reversed_with_copies));
}
