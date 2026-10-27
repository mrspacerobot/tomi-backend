//! Cook entries as the phone sends them: one entry read and checked at the edge, and its answer.

mod answer;
mod db;
mod dish;
mod entry;
mod fields;
mod problems;
mod request;
mod routes;
mod zone;

#[cfg(test)]
mod tests;

pub use answer::{EntryResult, EntryStatus};
pub use dish::Dish;
pub use entry::{id_of, EntryChecks, EntryOutcome, ReceivedEntry, ENTRY_SCHEMA_VERSION};
pub use problems::EntryProblem;
pub use request::{CheckedRequest, EntryRequest, RequestCalendar};

pub use routes::router;
