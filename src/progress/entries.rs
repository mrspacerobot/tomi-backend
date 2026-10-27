//! The cook entries as the rules read them: any list, in any order, possibly with copies, and the
//! finished cooks they count per skill and level, with the quiz result derived from the recorded first answers.

mod counts;
mod entry;
mod quiz;
mod stored;

#[cfg(test)]
mod tests;

pub use counts::CookCounts;
pub use entry::{finished_cooks, CookEntry, Outcome};
pub use quiz::QuizAnswers;
pub use stored::{StoredEntry, UnreadableStoredEntry};
