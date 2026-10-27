//! The weekly streak and the calendar it was counted in, as the snapshot sends them.

use super::SnapshotInputs;
use crate::progress::{CookCalendar, CookEntry, StreakRules};
use jiff::Timestamp;
use serde::Serialize;

/// The calendar of the request, as it was read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarAnswer {
    pub time_zone: String,
    pub first_weekday: i8,
}

impl CalendarAnswer {
    pub fn of(calendar: &CookCalendar) -> CalendarAnswer {
        CalendarAnswer {
            time_zone: calendar.time_zone_name().to_string(),
            first_weekday: calendar.first_weekday_number(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreakAnswer {
    /// Met weeks in a row, the week in progress included once it is met.
    pub weeks: u32,
    pub weeks_before_this_week: u32,
    pub days_this_week: u32,
    pub goal: u8,
    /// Today included.
    pub days_left_in_week: u32,
    pub this_week_met: bool,
    /// Still possible, but only by cooking on every day left.
    pub at_risk: bool,
    /// Too few days left to meet the goal.
    pub lost: bool,
    /// The first moment of the week in progress, in the request's zone.
    pub week_starts_at: Timestamp,
    /// The first moment of the next week: a kept snapshot's week is over from then.
    pub week_ends_at: Timestamp,
}

impl StreakAnswer {
    pub fn compute(log: &[CookEntry], inputs: &SnapshotInputs) -> StreakAnswer {
        let streak = StreakRules::new(inputs.calendar, inputs.answers.weekly_goal).streak(log, inputs.now);
        let week = inputs.calendar.week_of(inputs.now);
        StreakAnswer {
            weeks: streak.weeks(),
            weeks_before_this_week: streak.weeks_before_this_week,
            days_this_week: streak.days_this_week,
            goal: streak.goal,
            days_left_in_week: streak.days_left_in_week,
            this_week_met: streak.is_this_week_met(),
            at_risk: streak.is_at_risk(),
            lost: streak.is_lost(),
            week_starts_at: week.starts_at,
            week_ends_at: week.ends_at,
        }
    }
}
