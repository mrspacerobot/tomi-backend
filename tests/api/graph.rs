use crate::helpers::{get, server_with, Setup};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::fs;
use std::path::PathBuf;
use tomi_api::config::GraphConfig;
use tomi_api::graph::files;
use tomi_api::graph::{sha256_hex, GraphFiles};
use uuid::Uuid;

const VERSION: &str = "2026.10.11-1102817";

fn files(channel: &str) -> GraphFiles {
    let serving = json!({ "facts": [], "step_text": { "proper_name_words": [] }, "recipes": {} });
    let things = json!({ "technique/sear": { "tier": "drafted" } });
    let artifact = json!({ "artifactSchemaVersion": 3, "channel": channel, "things": things, "serving": serving });
    let artifact = artifact.to_string();
    let manifest = json!({
        "artifactSchemaVersion": 3, "graphVersion": VERSION, "sourceCommit": "1102817",
        "channel": channel, "contentSha256": sha256_hex(&artifact),
    });
    GraphFiles { manifest: manifest.to_string(), artifact }
}

fn lock_for(files: &GraphFiles) -> String {
    format!("version {VERSION}\nsha256 {}\n", sha256_hex(&files.artifact))
}

fn folder_with(files: &GraphFiles) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("tomi-graph-{}", Uuid::new_v4()));
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("manifest.json"), &files.manifest).unwrap();
    fs::write(folder.join("graph-artifact.json"), &files.artifact).unwrap();
    folder
}

fn config(folder: Option<&PathBuf>, allow_draft: bool) -> GraphConfig {
    GraphConfig { dir: folder.map(|path| path.to_string_lossy().to_string()), allow_draft, required: false }
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_names_the_graph_version_and_channel(pool: PgPool) {
    // The fixture is a real excerpt, so the shipped restrictions agree with it, as the start-up check demands.
    let app = server_with(pool, Setup::default());
    let reply = get(&app, "/v1/bootstrap", None).await;
    assert_eq!(reply.body["graphArtifactVersion"], "fixture");
    assert_eq!(reply.body["graphChannel"], "draft");
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_says_null_without_a_graph(pool: PgPool) {
    let app = server_with(pool, Setup { graph: None, ..Setup::default() });
    let reply = get(&app, "/v1/bootstrap", None).await;
    assert_eq!(reply.body["graphArtifactVersion"], Value::Null);
    assert_eq!(reply.body["graphChannel"], Value::Null);
}

#[test]
fn a_build_that_pins_no_graph_starts_without_one() {
    assert!(files::start("none\n", &config(None, false)).unwrap().is_none());
}

#[test]
fn a_build_that_pins_no_graph_does_not_start_when_a_graph_is_required() {
    let required = GraphConfig { required: true, ..config(None, false) };
    let error = files::start("none\n", &required).unwrap_err();
    assert!(error.contains("GRAPH_REQUIRED"), "{error}");
}

#[test]
fn a_required_graph_that_is_pinned_and_present_starts() {
    let files = files("draft");
    let folder = folder_with(&files);
    let required = GraphConfig { required: true, ..config(Some(&folder), true) };
    assert!(files::start(&lock_for(&files), &required).unwrap().is_some());
}

#[test]
fn a_pinned_graph_loads_from_its_folder() {
    let files = files("draft");
    let folder = folder_with(&files);
    let graph = files::start(&lock_for(&files), &config(Some(&folder), true)).unwrap().unwrap();
    assert_eq!(graph.version, VERSION);
    assert_eq!(graph.len(), 1);
}

#[test]
fn a_pinned_graph_without_a_folder_stops_the_start() {
    let files = files("draft");
    let error = files::start(&lock_for(&files), &config(None, true)).unwrap_err();
    assert!(error.contains("GRAPH_DIR"), "{error}");
}

#[test]
fn a_pinned_graph_that_is_missing_or_changed_stops_the_start() {
    let files = files("draft");
    let empty = std::env::temp_dir().join(format!("tomi-graph-{}", Uuid::new_v4()));
    fs::create_dir_all(&empty).unwrap();
    assert!(files::start(&lock_for(&files), &config(Some(&empty), true)).is_err());
    let other = self::files("production");
    let folder = folder_with(&other);
    assert!(files::start(&lock_for(&files), &config(Some(&folder), true)).is_err());
}

#[test]
fn a_lock_that_cannot_be_read_stops_the_start() {
    assert!(files::start("", &config(None, true)).is_err());
}

#[test]
fn a_draft_graph_is_refused_without_the_switch() {
    let files = files("draft");
    let folder = folder_with(&files);
    assert!(files::start(&lock_for(&files), &config(Some(&folder), false)).is_err());
}
