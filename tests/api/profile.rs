use crate::helpers::{get, new_cook, patch_profile, server};
use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn a_new_cook_starts_with_version_zero_and_defaults(pool: PgPool) {
    let reply = get(&server(pool), "/v1/profile", Some(&new_cook())).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["version"], 0);
    assert_eq!(reply.body["profile"]["diet"], Value::Null);
    assert_eq!(reply.body["profile"]["avoid"], json!([]));
    assert_eq!(reply.body["profile"]["weeklyGoal"], 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_saved_profile_reads_back_at_the_next_version(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let body = json!({"version": 0, "diet": "vegetarian", "avoid": ["nuts"], "units": "cups"});
    let saved = patch_profile(&app, &cook, body).await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    assert_eq!(saved.body["version"], 1);

    let read = get(&app, "/v1/profile", Some(&cook)).await;
    assert_eq!(read.body["version"], 1);
    assert_eq!(read.body["profile"]["diet"], "vegetarian");
    assert_eq!(read.body["profile"]["avoid"], json!(["nuts"]));
    assert_eq!(read.body["profile"]["units"], "cups");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stale_version_is_refused_with_the_current_profile(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    patch_profile(&app, &cook, json!({"version": 0, "servings": 3})).await;

    for stale in [0, 5] {
        let reply = patch_profile(&app, &cook, json!({"version": stale, "servings": 6})).await;
        assert_eq!(reply.status, StatusCode::CONFLICT);
        assert_eq!(reply.body["error"]["code"], "version_conflict");
        let current = &reply.body["error"]["details"]["current"];
        assert_eq!(current["version"], 1);
        assert_eq!(current["profile"]["servings"], 3);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn restrictions_are_never_dropped_silently(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    patch_profile(&app, &cook, json!({"version": 0, "avoid": ["nuts", "sesame"]})).await;

    let reply = patch_profile(&app, &cook, json!({"version": 1, "avoid": ["sesame"]})).await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert_eq!(reply.body["error"]["code"], "weakens_restrictions");
    assert_eq!(reply.body["error"]["details"]["removed"], json!(["avoid: nuts"]));
    let unchanged = get(&app, "/v1/profile", Some(&cook)).await;
    assert_eq!(unchanged.body["profile"]["avoid"], json!(["nuts", "sesame"]));

    let confirmed = json!({"version": 1, "avoid": ["sesame"], "allowWeakerRestrictions": true});
    let reply = patch_profile(&app, &cook, confirmed).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["profile"]["avoid"], json!(["sesame"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn bad_bodies_are_refused(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let bodies = [
        json!({"version": 0, "xp": 100}),
        json!({"servings": 2}),
        json!({"version": 0, "avoid": ["Nuts"]}),
        json!({"version": 0, "weeklyGoal": 9}),
        json!({"version": 0, "cuisinesLoved": ["thai"], "cuisinesNever": ["thai"]}),
    ];
    for body in bodies {
        let reply = patch_profile(&app, &cook, body.clone()).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(reply.body["error"]["code"], "invalid_request");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn cooks_see_only_their_own_profile(pool: PgPool) {
    let (app, ana, ben) = (server(pool), new_cook(), new_cook());
    patch_profile(&app, &ana, json!({"version": 0, "avoid": ["dairy"]})).await;

    let reply = get(&app, "/v1/profile", Some(&ben)).await;
    assert_eq!(reply.body["version"], 0);
    assert_eq!(reply.body["profile"]["avoid"], json!([]));
}
