//! Counts the weekly streak from the cook log at one moment, in the cook's calendar.

use super::weeks::CookingWeeks;
use super::{CookCalendar, Streak};
use crate::progress::game::clamped_weekly_goal;
use crate::progress::CookEntry;
use jiff::civil::Date;
use jiff::Timestamp;

/// The streak rules for one request: the cook's calendar and their current weekly goal.
pub struct StreakRules<'a> {
    calendar: &'a CookCalendar,
    goal: u8,
}

impl<'a> StreakRules<'a> {
    pub fn new(calendar: &'a CookCalendar, goal: u8) -> Self {
        StreakRules { calendar, goal }
    }

    /// The streak at `now`, the moment of the request. Entries may come in any order and may repeat.
    pub fn streak(&self, log: &[CookEntry], now: Timestamp) -> Streak {
        let today = self.calendar.day_of(now);
        let this_week = self.calendar.first_day_of_week(today);
        let weeks = CookingWeeks::new(log, self.calendar, this_week);
        Streak {
            weeks_before_this_week: self.met_weeks_in_a_row_before(this_week, &weeks),
            days_this_week: weeks.days_cooked(this_week),
            goal: clamped_weekly_goal(self.goal),
            days_left_in_week: self.calendar.days_left_in_week(today),
        }
    }

    /// Met weeks counted back from last week until the first week that was not met. The week in
    /// progress is not one of them, so it never breaks the streak before it ends.
    fn met_weeks_in_a_row_before(&self, this_week: Date, weeks: &CookingWeeks) -> u32 {
        let mut count = 0;
        let mut week = self.calendar.first_day_of_week_before(this_week);
        while weeks.was_goal_met(week) {
            count += 1;
            let earlier = self.calendar.first_day_of_week_before(week);
            if earlier == week {
                break;
            }
            week = earlier;
        }
        count
    }
}
