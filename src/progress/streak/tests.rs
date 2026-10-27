//! Tests of the weekly streak. Ported from TomiCore's `StreakRulesTests`, `StreakCalendarTests`,
//! `CookLogStreakTests` and `GameStateTests`, plus the server's own promises from issue #37: daylight
//! saving, the first weekday, and which zone decides a cook's day.

mod calendar;
mod cook_log;
mod daylight_saving;
mod rules;
mod test_calendar;
mod time_zones;
