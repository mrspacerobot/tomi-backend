//! Cook entries as the phone sends them: a batch read once at the edge, each entry checked into a type that
//! cannot hold a bad value, and the answer for each.

mod answer;
mod batch;
mod dish;
mod entry;
mod fields;
mod problems;
mod zone;

#[cfg(test)]
mod tests;

pub use answer::{EntryResult, EntryStatus};
pub use batch::{BatchFile, EntryBatch, MAX_ENTRIES_PER_BATCH};
pub use dish::Dish;
pub use entry::{id_of, EntryChecks, EntryOutcome, ReceivedEntry, ENTRY_SCHEMA_VERSION};
pub use problems::EntryProblem;
