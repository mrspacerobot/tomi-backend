//! Tests of the progress snapshot: what it holds, the documented example, and the week it reports.

mod answers;
mod example;
mod week;

use crate::kernel::language::Language;
use crate::library::tests::shipped_files;
use crate::library::{Library, LibraryFiles};
use crate::profile::Experience;
use crate::progress::game::WEEKLY_GOAL_DEFAULT;
use crate::progress::{CookCalendar, CookEntry, ProgressAnswers, ProgressSnapshot, SnapshotInputs};
use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::Timestamp;
use serde_json::Value;

/// A request for a snapshot against the shipped library: a calendar, a moment, and the cook's answers, which are
/// a new cook's unless a test says otherwise.
pub struct Request {
    pub library: Library,
    pub calendar: CookCalendar,
    pub answers: ProgressAnswers,
    pub language: Language,
    pub now: Timestamp,
}

impl Request {
    /// A request at `now` from a phone in this IANA zone whose weeks start on this weekday, 1 for Sunday.
    pub fn new(zone: &str, first_weekday: i64, now: Timestamp) -> Self {
        Request {
            library: Library::parse(&shipped_files()).unwrap(),
            calendar: CookCalendar::parse(zone, first_weekday).unwrap(),
            answers: ProgressAnswers {
                experience: Experience::Basics,
                weekly_goal: WEEKLY_GOAL_DEFAULT,
                focus_skill: None,
            },
            language: Language::english(),
            now,
        }
    }

    /// The same request against a library read from these files.
    pub fn with_library(self, files: &LibraryFiles) -> Self {
        Request { library: Library::parse(files).unwrap(), ..self }
    }

    pub fn snapshot(&self, log: &[CookEntry]) -> ProgressSnapshot {
        let inputs = SnapshotInputs {
            catalog: &self.library.catalog,
            rank_names: &self.library.ranks,
            language: &self.language,
            answers: &self.answers,
            calendar: &self.calendar,
            now: self.now,
        };
        ProgressSnapshot::compute(log, &inputs)
    }

    /// The snapshot as the phone reads it.
    pub fn json(&self, log: &[CookEntry]) -> Value {
        serde_json::to_value(self.snapshot(log)).unwrap()
    }
}

/// This moment of this day in UTC, to the second.
pub fn utc(day: Date, (hour, minute, second): (i8, i8, i8)) -> Timestamp {
    day.at(hour, minute, second, 0).to_zoned(TimeZone::UTC).unwrap().timestamp()
}
