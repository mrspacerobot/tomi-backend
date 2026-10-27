//! Answers belong to one Supabase account, and nothing a cook picks reaches the logs.

use super::*;
use crate::logs::LogBuffer;
use tracing::instrument::WithSubscriber;

#[sqlx::test(migrations = "./migrations")]
async fn one_cooks_answers_are_invisible_to_another_with_the_same_entry_id(pool: PgPool) {
    let app = server(pool.clone());
    let entry = Uuid::new_v4();
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    answer(&app, &token_with(&claims(first)), &pick(entry, QUESTION, WRONG)).await;
    let reply = answer(&app, &token_with(&claims(second)), &pick(entry, QUESTION, RIGHT)).await;
    assert_eq!(reply.body["scored"], true);
    assert_eq!(reply.body["correct"], true);
    let query = "select correct from tomi.quiz_answers where user_id = $1";
    for (subject, correct) in [(first, false), (second, true)] {
        let user = user_id(&pool, subject).await;
        let rows: Vec<bool> = sqlx::query_scalar(query).bind(user).fetch_all(&pool).await.unwrap();
        assert_eq!(rows, vec![correct]);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_account_is_refused_and_no_token_is_unauthorised(pool: PgPool) {
    let app = server(pool.clone());
    let subject = Uuid::new_v4();
    let token = token_with(&claims(subject));
    assert_eq!(answer(&app, &token, &pick(Uuid::new_v4(), QUESTION, RIGHT)).await.status, StatusCode::OK);
    let delete = "update tomi.users set deleted_at = now() where auth_subject = $1";
    sqlx::query(delete).bind(subject).execute(&pool).await.unwrap();
    let reply = answer(&app, &token, &pick(Uuid::new_v4(), QUESTION, RIGHT)).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "account_deleted");
    assert_eq!(stored(&pool).await.len(), 1);
    let body = Some(("application/json", pick(Uuid::new_v4(), QUESTION, RIGHT).to_string().into_bytes()));
    let anonymous = call(&app, Call { method: Method::POST, path: "/v1/quiz-answers", token: None, body }).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_account_takes_its_answers_with_it(pool: PgPool) {
    let app = server(pool.clone());
    let subject = Uuid::new_v4();
    answer(&app, &token_with(&claims(subject)), &pick(Uuid::new_v4(), QUESTION, RIGHT)).await;
    assert_eq!(stored(&pool).await.len(), 1);
    let erase = "delete from tomi.users where auth_subject = $1";
    sqlx::query(erase).bind(subject).execute(&pool).await.unwrap();
    assert!(stored(&pool).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn no_part_of_a_pick_or_its_answer_reaches_the_logs(pool: PgPool) {
    let app = server(pool);
    let (token, entry) = (new_cook(), Uuid::new_v4());
    let buffer = LogBuffer::default();
    let picks = async {
        answer(&app, &token, &pick(entry, QUESTION, WRONG)).await;
        answer(&app, &token, &pick(entry, QUESTION, RIGHT)).await;
        answer(&app, &token, &pick(entry, "technique/roll-an-omelette/quiz/purpose", 2)).await
    };
    let refused = picks.with_subscriber(buffer.subscriber()).await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    let logs = buffer.text();
    assert!(logs.contains("/v1/quiz-answers"), "the capture saw the requests: {logs}");
    for secret in [entry.to_string(), RECIPE.into(), "quiz/purpose".into(), "roll-an-omelette".into(), explanation(0)] {
        assert!(!logs.contains(&secret), "{secret} reached the logs: {logs}");
    }
}
