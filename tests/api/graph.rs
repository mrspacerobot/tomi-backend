use crate::helpers::{get, server_with, Setup};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::fs;
use std::path::PathBuf;
use tomi_api::config::GraphConfig;
use tomi_api::domain::graph::{sha256_hex, Graph, GraphFiles, GraphPin, GraphPolicy};
use tomi_api::graph_files;
use uuid::Uuid;

const VERSION: &str = "2026.10.11-1102817";

fn files(channel: &str) -> GraphFiles {
    let artifact = json!({ "artifactSchemaVersion": 1, "channel": channel, "things": { "technique/sear": { "tier": "drafted" } } });
    let artifact = artifact.to_string();
    let manifest = json!({
        "artifactSchemaVersion": 1, "graphVersion": VERSION, "sourceCommit": "1102817",
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
    GraphConfig { dir: folder.map(|path| path.to_string_lossy().to_string()), allow_draft }
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_names_the_graph_version_and_channel(pool: PgPool) {
    let files = files("draft");
    let pin = GraphPin::parse(&lock_for(&files)).unwrap().unwrap();
    let graph = Graph::load(&files, &pin, &GraphPolicy { allow_draft: true }).unwrap();
    let app = server_with(pool, Setup { graph: Some(graph), ..Setup::default() });
    let reply = get(&app, "/v1/bootstrap", None).await;
    assert_eq!(reply.body["graphArtifactVersion"], VERSION);
    assert_eq!(reply.body["graphChannel"], "draft");
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_says_null_without_a_graph(pool: PgPool) {
    let reply = get(&crate::helpers::server(pool), "/v1/bootstrap", None).await;
    assert_eq!(reply.body["graphArtifactVersion"], Value::Null);
    assert_eq!(reply.body["graphChannel"], Value::Null);
}

#[test]
fn a_build_that_pins_no_graph_starts_without_one() {
    assert!(graph_files::start("none\n", &config(None, false)).unwrap().is_none());
}

#[test]
fn a_pinned_graph_loads_from_its_folder() {
    let files = files("draft");
    let folder = folder_with(&files);
    let graph = graph_files::start(&lock_for(&files), &config(Some(&folder), true)).unwrap().unwrap();
    assert_eq!(graph.version, VERSION);
    assert_eq!(graph.len(), 1);
}

#[test]
fn a_pinned_graph_without_a_folder_stops_the_start() {
    let files = files("draft");
    let error = graph_files::start(&lock_for(&files), &config(None, true)).unwrap_err();
    assert!(error.contains("GRAPH_DIR"), "{error}");
}

#[test]
fn a_pinned_graph_that_is_missing_or_changed_stops_the_start() {
    let files = files("draft");
    let empty = std::env::temp_dir().join(format!("tomi-graph-{}", Uuid::new_v4()));
    fs::create_dir_all(&empty).unwrap();
    assert!(graph_files::start(&lock_for(&files), &config(Some(&empty), true)).is_err());
    let other = self::files("production");
    let folder = folder_with(&other);
    assert!(graph_files::start(&lock_for(&files), &config(Some(&folder), true)).is_err());
}

#[test]
fn a_lock_that_cannot_be_read_stops_the_start() {
    assert!(graph_files::start("", &config(None, true)).is_err());
}

#[test]
fn a_draft_graph_is_refused_without_the_switch() {
    let files = files("draft");
    let folder = folder_with(&files);
    assert!(graph_files::start(&lock_for(&files), &config(Some(&folder), false)).is_err());
}
