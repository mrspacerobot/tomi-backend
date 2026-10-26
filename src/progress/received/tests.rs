//! Tests of reading one received cook entry: its time limit, its fingerprint and its dish.

use super::{EntryChecks, EntryOutcome, EntryProblem, EntryStatus, ReceivedEntry};
use jiff::Timestamp;
use serde_json::{json, Value};

fn sent() -> Value {
    json!({
        "schemaVersion": 1, "id": "4e2c2a0e-7e6b-4c7a-9c2b-6e0a3d1f9a11", "cookedAt": "2026-10-26T18:42:00Z",
        "timeZone": "Europe/Amsterdam", "dish": {"lessonId": "eggs-scrambled-1"}, "skill": "eggs", "level": 1,
        "outcome": "finished", "weeklyGoal": 3,
    })
}

fn with(field: &str, value: Value) -> Value {
    let mut changed = sent();
    changed[field] = value;
    changed
}

fn received_at(text: &str) -> EntryChecks<'static> {
    EntryChecks { received_at: text.parse::<Timestamp>().unwrap(), graph: None }
}

fn read(raw: &Value) -> Result<ReceivedEntry, Vec<EntryProblem>> {
    ReceivedEntry::read(raw, &received_at("2026-10-26T18:42:00Z"))
}

#[test]
fn a_cook_up_to_the_tolerance_after_its_arrival_is_accepted_and_one_second_more_is_not() {
    let arrival = received_at("2026-10-26T18:32:00Z");
    assert!(ReceivedEntry::read(&sent(), &arrival).is_ok());
    let later = with("cookedAt", json!("2026-10-26T18:42:01Z"));
    assert_eq!(ReceivedEntry::read(&later, &arrival), Err(vec![EntryProblem::CookedInFuture]));
}

#[test]
fn a_read_entry_keeps_what_was_sent() {
    let entry = read(&sent()).unwrap();
    assert_eq!(entry.skill.as_str(), "eggs");
    assert_eq!((entry.level, entry.weekly_goal, entry.outcome), (1, 3, EntryOutcome::Finished));
    assert_eq!(entry.time_zone, "Europe/Amsterdam");
}

#[test]
fn the_fingerprint_is_the_same_however_the_instant_is_written() {
    let in_utc = read(&sent()).unwrap().payload_sha256().unwrap();
    let with_offset = read(&with("cookedAt", json!("2026-10-26T19:42:00+01:00"))).unwrap();
    assert_eq!(with_offset.payload_sha256().unwrap(), in_utc);
}

#[test]
fn the_fingerprint_changes_with_any_field() {
    let original = read(&sent()).unwrap().payload_sha256().unwrap();
    let changes = [
        with("outcome", json!("stoppedEarly")),
        with("level", json!(2)),
        with("weeklyGoal", json!(4)),
        with("timeZone", json!("Asia/Tokyo")),
        with("dish", json!({"lessonId": "eggs-scrambled-2"})),
        with("cookedAt", json!("2026-10-26T18:41:00Z")),
    ];
    for changed in changes {
        assert_ne!(read(&changed).unwrap().payload_sha256().unwrap(), original, "{changed}");
    }
}

#[test]
fn a_copy_is_a_duplicate_only_when_its_fingerprint_matches() {
    assert_eq!(EntryStatus::of_copy("same", "same"), EntryStatus::Duplicate);
    assert_eq!(EntryStatus::of_copy("first", "second"), EntryStatus::Conflict);
}

#[test]
fn a_dish_must_be_exactly_one_recipe_with_its_graph_version_or_one_lesson() {
    let unreadable = [
        json!({}),
        json!({"recipeId": "recipe/omelette"}),
        json!({"graphVersion": "2026.10.20"}),
        json!({"lessonId": "a", "recipeId": "recipe/omelette", "graphVersion": "2026.10.20"}),
        json!({"lessonId": ""}),
    ];
    for dish in unreadable {
        assert_eq!(read(&with("dish", dish.clone())), Err(vec![EntryProblem::UnreadableEntry]), "{dish}");
    }
}

#[test]
fn an_entry_without_a_schema_version_is_not_read() {
    let mut raw = sent();
    raw.as_object_mut().unwrap().remove("schemaVersion");
    assert_eq!(read(&raw), Err(vec![EntryProblem::UnsupportedSchemaVersion]));
}

#[test]
fn every_broken_rule_is_named_in_order() {
    let mut raw = with("skill", json!("Eggs"));
    raw["level"] = json!(0);
    raw["weeklyGoal"] = json!(8);
    raw["timeZone"] = json!("Nowhere/Town");
    let expected = vec![
        EntryProblem::InvalidSkillId,
        EntryProblem::LevelOutOfRange,
        EntryProblem::WeeklyGoalOutOfRange,
        EntryProblem::InvalidTimeZone,
    ];
    assert_eq!(read(&raw), Err(expected));
}
