//! The weekly streak: weeks in a row in which the cook cooked on at least their goal's number of
//! days, counted in the cook's own calendar. Weekly, not daily, because few people cook every day.
//!
//! A port of TomiCore's `StreakRules`, `CookingWeeks` and `Calendar+Week`, with one change the
//! server needs: each cook entry carries the zone it was made in. A week before the current one
//! counts each cook on its day in that zone; the week in progress counts it on its day in the zone
//! the request names.

mod calendar;
mod rules;
mod standing;
mod weeks;

#[cfg(test)]
mod tests;

pub use calendar::{CookCalendar, ParseCalendarError, WeekBounds};
pub use rules::StreakRules;
pub use standing::Streak;
