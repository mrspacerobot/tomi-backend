//! The cook entries as the rules read them: any list, in any order, possibly with copies, and the
//! finished cooks they count per skill and level.

mod counts;
mod entry;

#[cfg(test)]
mod tests;

pub use counts::CookCounts;
pub use entry::{finished_cooks, CookEntry, Outcome};
