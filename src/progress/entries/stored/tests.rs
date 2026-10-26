//! Tests of how a stored cook entry is read back for the rules.

use super::StoredCook;
use crate::progress::fixtures::id;
use crate::progress::{CookEntry, Outcome};
use uuid::Uuid;

fn stored(skill: &str, level: i16, outcome: &str) -> StoredCook {
    StoredCook { id: Uuid::nil(), skill: skill.into(), level, outcome: outcome.into() }
}

#[test]
fn a_finished_cook_is_read_without_a_quiz_bonus_until_answers_are_recorded() {
    let expected = CookEntry {
        id: Uuid::nil(),
        skill: id("eggs"),
        level: 2,
        outcome: Outcome::Finished { quiz_passed: false },
    };
    assert_eq!(stored("eggs", 2, "finished").read(), Some(expected));
}

#[test]
fn a_stopped_early_cook_is_read_as_stopped_early() {
    let entry = stored("eggs", 1, "stoppedEarly").read().unwrap();
    assert_eq!(entry.outcome, Outcome::StoppedEarly);
}

#[test]
fn a_row_the_rules_cannot_read_is_left_out() {
    assert_eq!(stored("Not An Id", 1, "finished").read(), None);
    assert_eq!(stored("eggs", 0, "finished").read(), None);
    assert_eq!(stored("eggs", 256, "finished").read(), None);
    assert_eq!(stored("eggs", 1, "abandoned").read(), None);
}
