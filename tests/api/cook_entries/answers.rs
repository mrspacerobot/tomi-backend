//! What the snapshot is read with: the request's calendar, the cook's profile answers and their language.

use super::*;
use axum::http::StatusCode;
use serde_json::json;
use std::path::Path;
use tomi_api::library::Library;

#[sqlx::test(migrations = "./migrations")]
async fn a_missing_or_unknown_calendar_is_refused(pool: PgPool) {
    let (app, cook) = (server_at_request_time(pool), new_cook());
    let refused = [
        "",
        "timeZone=Europe/Amsterdam",
        "firstWeekday=2",
        "timeZone=Mars/Olympus_Mons&firstWeekday=2",
        "timeZone=%2B02:00&firstWeekday=2",
        "timeZone=&firstWeekday=2",
        "timeZone=Europe/Amsterdam&firstWeekday=0",
        "timeZone=Europe/Amsterdam&firstWeekday=8",
        "timeZone=Europe/Amsterdam&firstWeekday=monday",
        "timeZone=Europe/Amsterdam&firstWeekday=2&weeklyGoal=5",
    ];
    for query in refused {
        let reply = read_progress(&app, Some(&cook), query).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{query}: {}", reply.body);
        assert_eq!(reply.body["error"]["code"], "invalid_request", "{query}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_token_progress_is_unauthenticated(pool: PgPool) {
    let reply = read_progress(&server_at_request_time(pool), None, AMSTERDAM).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn experience_comfortable_grants_the_first_course_with_no_xp(pool: PgPool) {
    let (app, cook) = (server_at_request_time(pool), new_cook());
    let saved = patch_profile(&app, &cook, json!({"version": 0, "experience": "comfortable"})).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);

    let progress = read_progress(&app, Some(&cook), AMSTERDAM).await.body;
    assert_eq!((&progress["xp"], &progress["rank"]["id"]), (&json!(0), &json!("novice")));
    let skills = progress["skills"].as_array().unwrap();
    for skill in
        skills.iter().filter(|skill| ["knife-cuts", "eggs", "stocks"].contains(&skill["skill"].as_str().unwrap()))
    {
        assert_eq!(skill["status"], "mastered", "{skill}");
        let levels = skill["levels"].as_array().unwrap();
        assert!(levels.iter().all(|level| level["granted"] == true && level["cooks"] == 0), "{skill}");
    }
    assert_eq!(progress["masteredCount"], 3);
    assert_eq!(progress["focusSkill"], "saute", "the first served skill past the granted course");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_changed_weekly_goal_or_focus_skill_shows_in_the_next_snapshot(pool: PgPool) {
    let (app, cook) = (server_at_request_time(pool), new_cook());
    let before = read_progress(&app, Some(&cook), AMSTERDAM).await.body;
    assert_eq!((&before["streak"]["goal"], &before["focusSkill"]), (&json!(3), &json!("knife-cuts")));

    let saved = patch_profile(&app, &cook, json!({"version": 0, "weeklyGoal": 6, "focusSkill": "eggs"})).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    let after = read_progress(&app, Some(&cook), AMSTERDAM).await.body;
    assert_eq!((&after["streak"]["goal"], &after["focusSkill"]), (&json!(6), &json!("eggs")));
    assert_eq!(after["streak"]["atRisk"], true, "six days needed and six left");
}

/// The shipped library, with the first rank also named in German.
fn library_with_german_rank_names() -> Library {
    let mut files = tomi_api::library::files::read(Path::new("library")).unwrap();
    let mut ranks: Value = serde_json::from_str(&files.ranks).unwrap();
    ranks["ranks"][0]["name"]["de"] = json!("Anfänger");
    files.ranks = ranks.to_string();
    Library::parse(&files).unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn the_rank_name_follows_accept_language_and_the_rank_id_does_not(pool: PgPool) {
    let clock = Arc::new(FixedClock(request_time()));
    let app = server_with(pool, Setup { library: library_with_german_rank_names(), clock, ..Setup::default() });
    let cook = new_cook();
    let path = format!("/v1/progress?{AMSTERDAM}");
    let in_language =
        |language| call_in(&app, Call { method: Method::GET, path: &path, token: Some(&cook), body: None }, language);
    let english = in_language("en").await.body;
    let german = in_language("de-DE,de;q=0.9").await.body;
    let french = in_language("fr").await.body;
    assert_eq!(english["rank"]["name"], "Novice");
    assert_eq!(german["rank"]["name"], "Anfänger");
    assert_eq!(french["rank"]["name"], "Novice", "a language without the name falls back to English");
    for answer in [&english, &german, &french] {
        assert_eq!(answer["rank"]["id"], "novice");
    }
}
