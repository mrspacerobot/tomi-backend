//! `docs/examples/progress.json` is what the rules compute, built with the real types. The app copies it verbatim
//! as its fixture, so it must never drift from the server.

use super::{utc, Request};
use crate::progress::fixtures::{cook, local, made_at, october, stopped_before_the_end, with_the_quiz_wrong};
use crate::progress::CookEntry;
use jiff::civil::Date;
use serde_json::Value;

/// A finished cook in Amsterdam at 7 pm. No quiz is passed: the quiz result is derived from recorded answers, which
/// come with mrspacerobot/tomi-backend#44.
fn cooked(skill: &str, level: u8, day: Date) -> CookEntry {
    with_the_quiz_wrong(&made_at(&cook(skill, level), &local("Europe/Amsterdam", day, 19)))
}

/// Three cooking days in each of the two weeks before the week of Monday 26 October, eggs mastered on the way,
/// one knife cut this week, and a cook stopped early that counts for nothing.
fn documented_log() -> Vec<CookEntry> {
    vec![
        cooked("eggs", 1, october(13)),
        cooked("eggs", 1, october(15)),
        cooked("eggs", 2, october(17)),
        cooked("eggs", 2, october(20)),
        cooked("eggs", 3, october(22)),
        cooked("eggs", 3, october(24)),
        stopped_before_the_end(&cooked("knife-cuts", 1, october(25))),
        cooked("knife-cuts", 1, october(26)),
    ]
}

#[test]
fn the_documented_example_is_what_the_rules_compute() {
    let documented: Value = serde_json::from_str(include_str!("../../../../docs/examples/progress.json")).unwrap();
    let request = Request::new("Europe/Amsterdam", 2, utc(october(27), (18, 43, 2)));
    assert_eq!(request.json(&documented_log()), documented);
}
