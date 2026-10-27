//! The game numbers the app needs to ask its questions, read from `progress::game` and never typed twice.

use crate::progress::game::{WEEKLY_GOAL_DEFAULT, WEEKLY_GOAL_MAXIMUM, WEEKLY_GOAL_MINIMUM};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownGame {
    weekly_goal: WeeklyGoal,
}

/// The cooking days a week the cook may pick as their streak goal, and the one onboarding starts at.
#[derive(Serialize)]
struct WeeklyGoal {
    minimum: u8,
    maximum: u8,
    default: u8,
}

impl ShownGame {
    pub fn from_rules() -> Self {
        let weekly_goal =
            WeeklyGoal { minimum: WEEKLY_GOAL_MINIMUM, maximum: WEEKLY_GOAL_MAXIMUM, default: WEEKLY_GOAL_DEFAULT };
        ShownGame { weekly_goal }
    }
}
