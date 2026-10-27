//! A fixed calendar for streak tests, as TomiCore's `StreakTestCalendar`, so they never depend on
//! the machine's clock, zone or locale. Weeks start on Monday in UTC unless a test says otherwise.

use crate::progress::fixtures::{cook, local, made_at, with_weekly_goal};
use crate::progress::{CookCalendar, CookEntry, Streak, StreakRules};
use jiff::civil::Date;
use jiff::Zoned;

/// The hour of a test cook, and of "now", unless a test says otherwise: 6 pm.
const EVENING: i8 = 18;

/// The goal of the streak tests unless a test says otherwise, as in TomiCore's tests.
pub const TEST_GOAL: u8 = 2;

#[derive(Clone)]
pub struct TestCalendar {
    zone: &'static str,
    pub calendar: CookCalendar,
    /// The cook's current weekly goal when the streak is asked for.
    goal: u8,
}

impl TestCalendar {
    pub fn new() -> Self {
        TestCalendar::with("UTC", 2)
    }

    /// A calendar in this IANA zone whose weeks start on this weekday, 1 for Sunday to 7 for Saturday.
    pub fn with(zone: &'static str, first_weekday: i64) -> Self {
        TestCalendar { zone, calendar: CookCalendar::parse(zone, first_weekday).unwrap(), goal: TEST_GOAL }
    }

    /// The same calendar, asked for the streak with this current goal.
    pub fn with_goal(&self, goal: u8) -> Self {
        TestCalendar { goal, ..self.clone() }
    }

    /// A moment on this day in this calendar's zone, at this hour.
    pub fn at(&self, day: Date, hour: i8) -> Zoned {
        local(self.zone, day, hour)
    }

    /// One finished cook at 6 pm on each of these days, made in this calendar's zone, with the test goal.
    pub fn cooks(&self, days: &[Date]) -> Vec<CookEntry> {
        self.cooks_with_goal(days, TEST_GOAL)
    }

    pub fn cooks_with_goal(&self, days: &[Date], goal: u8) -> Vec<CookEntry> {
        let made: Vec<CookEntry> = days.iter().map(|day| made_at(&cook("knife", 1), &self.at(*day, EVENING))).collect();
        made.iter().map(|entry| with_weekly_goal(entry, goal)).collect()
    }

    /// The streak at 6 pm on `day` in this calendar.
    pub fn streak(&self, log: &[CookEntry], day: Date) -> Streak {
        self.streak_at(log, &self.at(day, EVENING))
    }

    /// The streak at exactly this moment.
    pub fn streak_at(&self, log: &[CookEntry], now: &Zoned) -> Streak {
        StreakRules::new(&self.calendar, self.goal).streak(log, now.timestamp())
    }
}
