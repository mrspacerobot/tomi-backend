//! XP, counted from the cook log every time it is asked for, never kept as a running total.

use crate::progress::game::{QUIZ_BONUS_XP, XP_PER_FINISHED_COOK};
use crate::progress::{finished_cooks, CookEntry, Outcome};

/// What a cook earns for the way it ended: the cook itself, plus the quiz bonus when the quiz was
/// passed. A cook stopped early earns nothing.
pub fn xp_for(outcome: Outcome) -> u32 {
    match outcome {
        Outcome::Finished { quiz_passed: true } => XP_PER_FINISHED_COOK + QUIZ_BONUS_XP,
        Outcome::Finished { quiz_passed: false } => XP_PER_FINISHED_COOK,
        Outcome::StoppedEarly => 0,
    }
}

/// The XP of every finished cook in the log. An entry stopped early earns nothing and a repeated id
/// earns once. A skill or level the catalog does not know still earns, as it does for the streak:
/// the cook did cook.
pub fn earned_xp(log: &[CookEntry]) -> u32 {
    let mut total: u32 = 0;
    for cook in finished_cooks(log) {
        total = total.saturating_add(xp_for(cook.outcome));
    }
    total
}
