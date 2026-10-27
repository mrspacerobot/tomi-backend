//! What a snapshot is computed from besides the cook entries.

use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::library::{Catalog, RankNames};
use crate::profile::{Experience, Profile};
use crate::progress::CookCalendar;
use jiff::Timestamp;

/// The cook's profile answers that progress depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressAnswers {
    /// "Comfortable" grants the first course.
    pub experience: Experience,
    /// The goal the week in progress is judged by.
    pub weekly_goal: u8,
    /// The skill the cook picked to cook next, if any.
    pub focus_skill: Option<SavedId>,
}

impl From<&Profile> for ProgressAnswers {
    fn from(profile: &Profile) -> Self {
        ProgressAnswers {
            experience: profile.experience,
            weekly_goal: profile.weekly_goal,
            focus_skill: profile.focus_skill.clone(),
        }
    }
}

/// The catalog, the words, the cook's answers and the request's calendar and moment.
pub struct SnapshotInputs<'a> {
    pub catalog: &'a Catalog,
    pub rank_names: &'a RankNames,
    /// The language of the request, for the rank's name.
    pub language: &'a Language,
    pub answers: &'a ProgressAnswers,
    pub calendar: &'a CookCalendar,
    /// The moment of the request. The rules never read a clock.
    pub now: Timestamp,
}
