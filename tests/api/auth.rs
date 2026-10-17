use crate::helpers::{claims, get, now, server, server_with, token_with, Setup, PROJECT};
use axum::http::StatusCode;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use p256::pkcs8::EncodePrivateKey;
use serde_json::{json, Value};
use sqlx::PgPool;
use tomi_api::auth::Verifier;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn a_request_without_a_token_is_refused(pool: PgPool) {
    let reply = get(&server(pool), "/v1/profile", None).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.body["error"]["code"], "unauthenticated");
}

#[sqlx::test(migrations = "./migrations")]
async fn expired_foreign_and_forged_tokens_are_refused(pool: PgPool) {
    let app = server(pool);
    let user = Uuid::new_v4();
    let with = |field: &str, value: Value| {
        let mut changed = claims(user);
        changed[field] = value;
        token_with(&changed)
    };
    let forged_key = EncodingKey::from_secret(b"guessed");
    let refused = [
        with("exp", json!(now() - 3600)),
        with("iss", json!("https://someone-else.supabase.co/auth/v1")),
        with("aud", json!("anon")),
        with("role", json!("anon")),
        encode(&Header::new(Algorithm::HS256), &claims(user), &forged_key).unwrap(),
        "junk".to_string(),
    ];
    for token in refused {
        let reply = get(&app, "/v1/profile", Some(&token)).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{token}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn anonymous_sessions_are_refused(pool: PgPool) {
    let mut anonymous = claims(Uuid::new_v4());
    anonymous["is_anonymous"] = json!(true);
    let reply = get(&server(pool), "/v1/profile", Some(&token_with(&anonymous))).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "anonymous_not_allowed");
}

#[sqlx::test(migrations = "./migrations")]
async fn asymmetric_tokens_are_checked_against_the_key_set(pool: PgPool) {
    let secret = p256::SecretKey::random(&mut rand_core::OsRng);
    let mut jwk: Value = serde_json::from_str(&secret.public_key().to_jwk_string()).unwrap();
    jwk["kid"] = json!("key-1");
    jwk["alg"] = json!("ES256");
    let jwks = serde_json::from_value(json!({ "keys": [jwk] })).unwrap();
    let app = server_with(pool, Setup { verifier: Verifier::with_keys(PROJECT, &jwks), ..Setup::default() });
    let signing = EncodingKey::from_ec_der(secret.to_pkcs8_der().unwrap().as_bytes());
    let sign = |kid: &str| {
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(kid.into());
        encode(&header, &claims(Uuid::new_v4()), &signing).unwrap()
    };

    let known = get(&app, "/v1/profile", Some(&sign("key-1"))).await;
    assert_eq!(known.status, StatusCode::OK);
    let unknown = get(&app, "/v1/profile", Some(&sign("key-unknown"))).await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_account_is_refused(pool: PgPool) {
    let app = server(pool.clone());
    let subject = Uuid::new_v4();
    let token = token_with(&claims(subject));
    assert_eq!(get(&app, "/v1/profile", Some(&token)).await.status, StatusCode::OK);

    let delete = "update tomi.users set deleted_at = now() where auth_subject = $1";
    sqlx::query(delete).bind(subject).execute(&pool).await.unwrap();
    let reply = get(&app, "/v1/profile", Some(&token)).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "account_deleted");
}
