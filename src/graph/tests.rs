use super::*;
use serde_json::json;

const VERSION: &str = "2026.10.11-1102817";

fn artifact_text(schema: u64, channel: &str) -> String {
    let things = json!({
        "technique/sear": { "tier": "drafted", "names": { "en": "Sear" } },
        "ingredient/egg": { "tier": "drafted", "safety": { "state": "unverified" } },
    });
    json!({ "artifactSchemaVersion": schema, "channel": channel, "things": things }).to_string()
}

fn manifest_text(artifact: &str, channel: &str) -> String {
    json!({
        "artifactSchemaVersion": 1,
        "graphVersion": VERSION,
        "sourceCommit": "1102817abcdef",
        "channel": channel,
        "contentSha256": sha256_hex(artifact),
        "builtAt": "2026-10-11T12:00:00Z",
        "counts": {},
    })
    .to_string()
}

fn files(channel: &str) -> GraphFiles {
    let artifact = artifact_text(1, channel);
    GraphFiles { manifest: manifest_text(&artifact, channel), artifact }
}

fn pin_for(files: &GraphFiles) -> GraphPin {
    GraphPin { version: VERSION.to_string(), sha256: sha256_hex(&files.artifact) }
}

const DRAFTS: GraphPolicy = GraphPolicy { allow_draft: true };
const NO_DRAFTS: GraphPolicy = GraphPolicy { allow_draft: false };

#[test]
fn a_graph_that_matches_its_pin_loads_into_memory() {
    let files = files("draft");
    let graph = Graph::load(&files, &pin_for(&files), &DRAFTS).unwrap();
    assert_eq!(graph.version, VERSION);
    assert_eq!(graph.channel, Channel::Draft);
    assert_eq!(graph.len(), 2);
    assert_eq!(graph.thing("technique/sear").unwrap()["tier"], "drafted");
    assert_eq!(graph.thing("technique/unknown"), None);
}

#[test]
fn an_unknown_schema_version_is_refused_before_anything_else_is_read() {
    let files = GraphFiles { manifest: json!({ "artifactSchemaVersion": 2 }).to_string(), artifact: "not json".into() };
    let pin = GraphPin { version: "x".into(), sha256: "y".into() };
    assert_eq!(Graph::load(&files, &pin, &DRAFTS).unwrap_err(), GraphError::UnsupportedSchema(2));
}

#[test]
fn a_manifest_that_cannot_be_read_is_refused() {
    let files = GraphFiles { manifest: "{".into(), artifact: String::new() };
    let pin = GraphPin { version: "x".into(), sha256: "y".into() };
    assert!(matches!(Graph::load(&files, &pin, &DRAFTS).unwrap_err(), GraphError::Unreadable(_)));
}

#[test]
fn a_changed_artifact_fails_its_checksum() {
    let mut files = files("draft");
    let pin = pin_for(&files);
    files.artifact.push(' ');
    assert_eq!(Graph::load(&files, &pin, &DRAFTS).unwrap_err(), GraphError::ChecksumMismatch);
}

#[test]
fn a_graph_that_is_not_the_pinned_one_is_refused() {
    let files = files("draft");
    let other = GraphPin { sha256: sha256_hex("another artifact"), ..pin_for(&files) };
    assert_eq!(Graph::load(&files, &other, &DRAFTS).unwrap_err(), GraphError::NotPinned);
    let other_version = GraphPin { version: "2026.01.01-abcdef0".into(), ..pin_for(&files) };
    assert_eq!(Graph::load(&files, &other_version, &DRAFTS).unwrap_err(), GraphError::NotPinned);
}

#[test]
fn a_draft_graph_needs_the_draft_switch() {
    let files = files("draft");
    assert_eq!(Graph::load(&files, &pin_for(&files), &NO_DRAFTS).unwrap_err(), GraphError::DraftRefused);
}

#[test]
fn a_production_graph_needs_no_switch() {
    let files = files("production");
    let graph = Graph::load(&files, &pin_for(&files), &NO_DRAFTS).unwrap();
    assert_eq!(graph.channel, Channel::Production);
}

#[test]
fn an_unknown_channel_is_refused_never_read_as_production() {
    let files = files("preview");
    let error = Graph::load(&files, &pin_for(&files), &DRAFTS).unwrap_err();
    assert_eq!(error, GraphError::UnknownChannel("preview".into()));
}

#[test]
fn a_manifest_that_claims_another_channel_than_its_artifact_is_refused() {
    let artifact = artifact_text(1, "draft");
    let files = GraphFiles { manifest: manifest_text(&artifact, "production"), artifact };
    let error = Graph::load(&files, &pin_for(&files), &NO_DRAFTS).unwrap_err();
    assert_eq!(error, GraphError::ChannelMismatch);
}

#[test]
fn an_artifact_of_another_schema_version_than_its_manifest_is_refused() {
    let artifact = artifact_text(2, "draft");
    let files = GraphFiles { manifest: manifest_text(&artifact, "draft"), artifact };
    assert_eq!(Graph::load(&files, &pin_for(&files), &DRAFTS).unwrap_err(), GraphError::UnsupportedSchema(2));
}

#[test]
fn an_artifact_that_is_not_json_is_refused_without_echoing_it() {
    let artifact = "secret text".to_string();
    let files = GraphFiles { manifest: manifest_text(&artifact, "draft"), artifact };
    let error = Graph::load(&files, &pin_for(&files), &DRAFTS).unwrap_err();
    assert!(matches!(error, GraphError::Unreadable(_)));
    assert!(!error.to_string().contains("secret"));
}

#[test]
fn the_pin_none_means_no_graph() {
    assert_eq!(GraphPin::parse("none\n").unwrap(), None);
    assert_eq!(GraphPin::parse("# no graph yet\nnone").unwrap(), None);
}

#[test]
fn a_pin_names_a_version_and_a_checksum() {
    let sum = sha256_hex("x");
    let text = format!("# pinned by scripts/fetch-graph.sh\nversion {VERSION}\nsha256 {sum}\n");
    let pin = GraphPin::parse(&text).unwrap().unwrap();
    assert_eq!(pin, GraphPin { version: VERSION.into(), sha256: sum });
}

#[test]
fn a_pin_that_cannot_be_read_is_an_error_never_no_graph() {
    let sum = sha256_hex("x");
    for text in ["", "version only\n", "version 1\n", &format!("sha256 {sum}\n"), "version 1\nsha256 abc\n"] {
        assert!(GraphPin::parse(text).is_err(), "{text:?}");
    }
    let twice = format!("version a\nversion b\nsha256 {sum}\n");
    assert!(GraphPin::parse(&twice).is_err());
}

#[test]
fn the_checksum_is_sha256_in_lowercase_hex() {
    let empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert_eq!(sha256_hex(""), empty);
}
