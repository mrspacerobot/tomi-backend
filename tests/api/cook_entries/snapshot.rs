//! `GET /v1/progress` and the cook-entries answer: the snapshot computed from the stored log on every request.

use super::*;
use axum::http::StatusCode;
use serde_json::json;

/// A finished cook of a graph recipe at 6 pm UTC, evening in Amsterdam, with the default weekly goal.
fn cooked(skill: &str, level: u8, day: &str) -> Value {
    let entry = with(&entry(), "dish", json!({"recipeId": "recipe/example", "graphVersion": "older"}));
    let entry = with(&with(&entry, "skill", json!(skill)), "level", json!(level));
    with(&entry, "cookedAt", json!(format!("2026-10-{day}T18:00:00Z")))
}

/// The log of `docs/examples/progress.json`: three cooking days in each of the two weeks before the week of
/// Monday 26 October, eggs mastered on the way, a cook stopped early, and one knife cut this week.
fn documented_log() -> Vec<Value> {
    let stopped = with(&cooked("knife-cuts", 1, "25"), "outcome", json!("stoppedEarly"));
    let days = [("eggs", 1, "13"), ("eggs", 1, "15"), ("eggs", 2, "17"), ("eggs", 2, "20"), ("eggs", 3, "22")];
    let mut log: Vec<Value> = days.iter().map(|(skill, level, day)| cooked(skill, *level, day)).collect();
    log.extend([cooked("eggs", 3, "24"), stopped, cooked("knife-cuts", 1, "26")]);
    log
}

fn documented_progress() -> Value {
    serde_json::from_str(include_str!("../../../docs/examples/progress.json")).unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_cook_who_posts_entries_reads_their_xp_rank_streak_and_skills(pool: PgPool) {
    let (app, cook) = (server_at_request_time(pool), new_cook());
    results(&app, &cook, &documented_log()).await;

    let reply = read_progress(&app, Some(&cook), AMSTERDAM).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let progress = &reply.body;
    assert_eq!(progress["xp"], 350, "seven finished lesson cooks of 50 XP: a lesson entry has no quiz");
    assert_eq!(progress["rank"], json!({"id": "commis", "name": "Commis", "tier": 1, "startsAt": 150, "nextAt": 400}));
    assert_eq!((&progress["streak"]["weeks"], &progress["streak"]["daysThisWeek"]), (&json!(2), &json!(1)));
    assert_eq!(
        (&progress["skills"][1]["skill"], &progress["skills"][1]["status"]),
        (&json!("eggs"), &json!("mastered"))
    );
    assert_eq!(progress["focusSkill"], "knife-cuts");
    assert_eq!(*progress, documented_progress());
}

#[sqlx::test(migrations = "./migrations")]
async fn the_same_snapshot_comes_back_from_get_progress_and_from_the_cook_entries_answer(pool: PgPool) {
    let (app, cook) = (server_at_request_time(pool), new_cook());
    let log = documented_log();
    results(&app, &cook, &log[..log.len() - 1]).await;
    let posted = send(&app, Some(&cook), &request(&log[log.len() - 1])).await;
    assert_eq!(posted.status, StatusCode::OK, "{}", posted.body);
    let read = read_progress(&app, Some(&cook), AMSTERDAM).await;
    assert_eq!(posted.body["progress"], read.body);

    results(&app, &cook, &documented_log()[..1]).await;
    let again = send(&app, Some(&cook), &request(&documented_log()[1])).await;
    assert_eq!(again.body["result"]["status"], "accepted", "each cooked() has a new id");
    let read_again = read_progress(&app, Some(&cook), AMSTERDAM).await;
    assert_eq!(again.body["progress"], read_again.body);
    assert_eq!(read_again.body["xp"], 450);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_cook_entry_answer_is_computed_in_the_requests_calendar(pool: PgPool) {
    let (app, cook) = (server_at_request_time(pool), new_cook());
    let sunday_first = json!({"calendar": {"timeZone": "America/New_York", "firstWeekday": 1}, "entry": entry()});
    let posted = send(&app, Some(&cook), &sunday_first).await;
    let read = read_progress(&app, Some(&cook), "timeZone=America/New_York&firstWeekday=1").await;
    assert_eq!(posted.body["progress"]["calendar"], json!({"timeZone": "America/New_York", "firstWeekday": 1}));
    assert_eq!(posted.body["progress"]["streak"]["weekStartsAt"], "2026-10-25T04:00:00Z");
    assert_eq!(posted.body["progress"], read.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_new_cook_gets_a_complete_snapshot_of_zeros_the_server_computed(pool: PgPool) {
    let reply = read_progress(&server_at_request_time(pool), Some(&new_cook()), AMSTERDAM).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let progress = &reply.body;
    let fields: Vec<&str> = progress.as_object().unwrap().keys().map(String::as_str).collect();
    let expected = ["calendar", "catalogVersion", "computedAt", "counted", "focusSkill", "masteredCount", "rank"];
    let expected = [&expected[..], &["rulesVersion", "schemaVersion", "skills", "streak", "xp"]].concat();
    assert_eq!(fields, expected);
    assert_eq!(
        (&progress["xp"], &progress["rank"]["id"], &progress["masteredCount"]),
        (&json!(0), &json!("novice"), &json!(0))
    );
    let streak = json!({
        "weeks": 0, "weeksBeforeThisWeek": 0, "daysThisWeek": 0, "goal": 3, "daysLeftInWeek": 6,
        "thisWeekMet": false, "atRisk": false, "lost": false,
        "weekStartsAt": "2026-10-25T23:00:00Z", "weekEndsAt": "2026-11-01T23:00:00Z",
    });
    assert_eq!(progress["streak"], streak);
    assert_eq!(progress["computedAt"], "2026-10-27T18:43:02Z");
    assert_eq!(progress["counted"], json!({"finishedCooks": 0, "ignoredUnknownSkillOrLevel": 0}));
    let skills = progress["skills"].as_array().unwrap();
    assert_eq!(skills.len(), 12);
    assert!(skills.iter().flat_map(|skill| skill["levels"].as_array().unwrap()).all(|level| level["cooks"] == 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn another_cooks_entries_never_reach_this_cooks_snapshot(pool: PgPool) {
    let (app, cook, other) = (server_at_request_time(pool), new_cook(), new_cook());
    results(&app, &other, &documented_log()).await;
    let reply = read_progress(&app, Some(&cook), AMSTERDAM).await;
    assert_eq!((&reply.body["xp"], &reply.body["counted"]["finishedCooks"]), (&json!(0), &json!(0)));
}
