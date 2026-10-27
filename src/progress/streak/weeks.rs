//! Finished cooks grouped into the cook's calendar weeks, each found by the day it starts on.

use super::CookCalendar;
use crate::kernel::calendar::day_in_zone;
use crate::progress::game::clamped_weekly_goal;
use crate::progress::{finished_cooks, CookEntry};
use jiff::civil::Date;
use jiff::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

/// One calendar week of the cook log: the days with a finished cook, and the goal the cook had.
#[derive(Debug, Default)]
struct CookingWeek {
    days_cooked: BTreeSet<Date>,
    /// The latest cook of the week and its goal. At the same instant the higher goal wins, so the
    /// order the entries arrive in never changes a week's goal.
    latest_cook: Option<(Timestamp, u8)>,
}

impl CookingWeek {
    fn add(&mut self, cook: &CookEntry, day: Date) {
        self.days_cooked.insert(day);
        let this_cook = (cook.cooked_at, cook.weekly_goal);
        let is_latest = match self.latest_cook {
            Some(latest) => this_cook > latest,
            None => true,
        };
        if is_latest {
            self.latest_cook = Some(this_cook);
        }
    }

    fn is_goal_met(&self) -> bool {
        let Some((_, goal)) = self.latest_cook else {
            return false;
        };
        self.days_cooked() >= u32::from(clamped_weekly_goal(goal))
    }

    fn days_cooked(&self) -> u32 {
        u32::try_from(self.days_cooked.len()).unwrap_or(u32::MAX)
    }
}

/// The finished cooks of a log in the weeks of the cook's calendar, as of the week in progress.
pub struct CookingWeeks {
    weeks_by_first_day: BTreeMap<Date, CookingWeek>,
}

impl CookingWeeks {
    /// Groups the finished cooks, each on the day `day_of_cook` gives it.
    pub fn new(log: &[CookEntry], calendar: &CookCalendar, this_week: Date) -> Self {
        let mut weeks_by_first_day: BTreeMap<Date, CookingWeek> = BTreeMap::new();
        for cook in finished_cooks(log) {
            let day = day_of_cook(cook, calendar, this_week);
            let week = weeks_by_first_day.entry(calendar.first_day_of_week(day)).or_default();
            week.add(cook, day);
        }
        CookingWeeks { weeks_by_first_day }
    }

    pub fn days_cooked(&self, first_day: Date) -> u32 {
        match self.weeks_by_first_day.get(&first_day) {
            Some(week) => week.days_cooked(),
            None => 0,
        }
    }

    /// A week with no cooks never meets its goal.
    pub fn was_goal_met(&self, first_day: Date) -> bool {
        match self.weeks_by_first_day.get(&first_day) {
            Some(week) => week.is_goal_met(),
            None => false,
        }
    }
}

/// The day a cook counts on. On the request's clocks it falls in the week in progress (or later)?
/// Then that is its day. Otherwise it belongs to the past, and counts on its day where it was made.
fn day_of_cook(cook: &CookEntry, calendar: &CookCalendar, this_week: Date) -> Date {
    let day_on_the_requests_clocks = calendar.day_of(cook.cooked_at);
    if day_on_the_requests_clocks >= this_week {
        return day_on_the_requests_clocks;
    }
    day_in_zone(cook.cooked_at, &cook.time_zone)
}
