//! A test server with fakes for the model and the photo store, and tokens.

use crate::fakes::{FakeAi, FakeStore};
use axum::Router;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tomi_api::ai::Ai;
use tomi_api::auth::Verifier;
use tomi_api::config::GraphConfig;
use tomi_api::graph::{files, Graph};
use tomi_api::library::Library;
use tomi_api::storage::PhotoStore;
use tomi_api::{app, AppState};
use uuid::Uuid;

pub use crate::client::{call, call_in, get, patch_profile, Call, Reply};
use axum::http::Method;

pub const PROJECT: &str = "https://test-project.supabase.co";
const SECRET: &str = "test-only-shared-secret-not-used-anywhere-else";

/// What a test server is built from. The default accepts tokens signed
/// with the test secret and uses a fake model and a fake photo store.
pub struct Setup {
    pub verifier: Verifier,
    pub ai: Arc<dyn Ai>,
    pub photos: Arc<dyn PhotoStore>,
    pub library: Library,
    pub graph: Option<Graph>,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            verifier: Verifier::new(PROJECT, Some(SECRET)).unwrap(),
            ai: Arc::new(FakeAi::default()),
            photos: Arc::new(FakeStore::default()),
            library: shipped_library(),
            graph: Some(fixture_graph()),
        }
    }
}

pub fn server(pool: PgPool) -> Router {
    server_with(pool, Setup::default())
}

pub fn server_with(pool: PgPool, setup: Setup) -> Router {
    let state = AppState {
        db: pool,
        verifier: Arc::new(setup.verifier),
        min_app_version: "1.2.0".into(),
        library: Arc::new(setup.library),
        graph: setup.graph.map(Arc::new),
        ai: setup.ai,
        photos: setup.photos,
    };
    app(state)
}

/// A small excerpt of a real graph release (`scripts/make-graph-fixture.py`), loaded as the server loads one.
pub fn fixture_graph() -> Graph {
    let config = GraphConfig { dir: Some("tests/fixtures/graph".to_string()), allow_draft: true, required: true };
    files::start(include_str!("../fixtures/graph/graph.lock"), &config).unwrap().unwrap()
}

fn shipped_library() -> Library {
    let files = tomi_api::library::files::read(Path::new("library")).unwrap();
    Library::parse(&files).unwrap().0
}

/// The shipped library with every prerequisite removed, so a comfortable
/// cook can reach every lesson. Tests about food use it; tests about
/// progress use the shipped one.
pub fn library_without_prerequisites() -> Library {
    let mut files = tomi_api::library::files::read(Path::new("library")).unwrap();
    let mut catalog: Value = serde_json::from_str(&files.catalog).unwrap();
    for skill in catalog["skills"].as_array_mut().unwrap() {
        skill["prerequisites"] = json!([]);
    }
    files.catalog = catalog.to_string();
    Library::parse(&files).unwrap().0
}

pub fn now() -> u64 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    since_epoch.as_secs()
}

/// The claims Supabase puts on a signed-in user's token.
pub fn claims(subject: Uuid) -> Value {
    json!({
        "sub": subject,
        "aud": "authenticated",
        "role": "authenticated",
        "iss": format!("{PROJECT}/auth/v1"),
        "exp": now() + 600,
        "is_anonymous": false,
    })
}

pub fn token_with(claims: &Value) -> String {
    let key = EncodingKey::from_secret(SECRET.as_bytes());
    encode(&Header::new(Algorithm::HS256), claims, &key).unwrap()
}

/// A valid token for a new cook.
pub fn new_cook() -> String {
    token_with(&claims(Uuid::new_v4()))
}

/// The cook's own id inside Tomi, once they have called the API.
pub async fn user_id(pool: &PgPool, subject: Uuid) -> Uuid {
    let query = "select id from tomi.users where auth_subject = $1";
    sqlx::query_scalar(query).bind(subject).fetch_one(pool).await.unwrap()
}

pub fn jpeg_with_location() -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xD8];
    bytes.extend([0xFF, 0xE0, 0x00, 0x04, b'J', b'F']);
    bytes.extend([0xFF, 0xE1, 0x00, 0x07, b'G', b'P', b'S', b'4', b'7']);
    bytes.extend([0xFF, 0xDA, 0x00, 0x02, 0x11, 0x22, 0xFF, 0xD9]);
    bytes
}

/// The photo the tests scan, with its content type.
pub fn jpeg() -> (&'static str, Vec<u8>) {
    ("image/jpeg", jpeg_with_location())
}

/// Scans a photo given as content type and bytes.
pub async fn scan(app: &Router, token: &str, upload: (&str, Vec<u8>)) -> Reply {
    let body = Some(upload);
    call(app, Call { method: Method::POST, path: "/v1/scans", token: Some(token), body }).await
}

pub async fn special(app: &Router, token: &str, body: Value) -> Reply {
    let body = Some(("application/json", body.to_string().into_bytes()));
    call(app, Call { method: Method::POST, path: "/v1/lessons/suggest", token: Some(token), body }).await
}
