//! Every number of the game, in one place, as Tomi's `GameRules`. No other file may type one.
//!
//! The streak is the everyday hook; rank is the slow, long goal, so it is paced in weeks and months,
//! not cooks. Pacing assumes a regular cook: about 60 XP a cook, three or four cooks a week.

use jiff::SignedDuration;

/// Bumped whenever a number here or a rule of this feature changes what a log is worth.
pub const RULES_VERSION: u32 = 1;

/// What a finished cook earns, before any quiz bonus.
pub const XP_PER_FINISHED_COOK: u32 = 50;

/// What a finished cook earns on top when its quiz was passed.
pub const QUIZ_BONUS_XP: u32 = 10;

/// XP at which each named rank begins, in `RankTitle` order. Each step is roughly twice the last:
/// Commis in the first week, Saucier after a month, Chef after four months, Toque after about a year.
pub const RANK_THRESHOLDS: [u32; 8] = [0, 150, 400, 900, 1_800, 3_300, 6_000, 11_000];

/// Past Toque the ladder never ends: a new tier (Toque II, III...) every this much XP, about one a season.
pub const XP_PER_TIER_PAST_TOP: u32 = 3_000;

/// The fewest cooking days a week the cook may set as their streak goal. Asked in onboarding.
pub const WEEKLY_GOAL_MINIMUM: u8 = 1;

/// The most cooking days a week the cook may set as their streak goal.
pub const WEEKLY_GOAL_MAXIMUM: u8 = 7;

/// The streak goal of a cook who has not picked one.
pub const WEEKLY_GOAL_DEFAULT: u8 = 3;

/// How far after its arrival a cook may say it was made, for a phone clock that runs a little fast. A cook later
/// than this is refused as `cooked_in_future`.
pub const COOKED_IN_FUTURE_TOLERANCE: SignedDuration = SignedDuration::from_secs(10 * 60);
/// The goal a week is judged by: the cook's goal, held inside the range they may pick from.
pub fn clamped_weekly_goal(goal: u8) -> u8 {
    goal.clamp(WEEKLY_GOAL_MINIMUM, WEEKLY_GOAL_MAXIMUM)
}

/// Finished cooks at one level of a skill before that level is complete
/// and the next one opens.
pub const COOKS_TO_COMPLETE_LEVEL: u32 = 2;

/// Whether this many finished cooks complete a level.
pub fn is_level_complete(cooks: u32) -> bool {
    cooks >= COOKS_TO_COMPLETE_LEVEL
}
