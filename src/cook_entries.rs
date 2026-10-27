//! Cook entries as the phone sends them: one entry read and checked at the edge, and its submission result.

mod db;
mod dish;
mod entry;
mod parsed_entry;
mod problems;
mod request;
mod response;
mod routes;
mod stored;
mod submission;

#[cfg(test)]
mod tests;

pub use dish::Dish;
pub use entry::{id_of, CookOutcome, EntryChecks, ValidatedCookEntry, ENTRY_SCHEMA_VERSION};
pub use problems::EntryProblem;
pub use request::{CheckedRequest, CookEntryRequest};
pub use response::{CookEntryResponse, SubmissionResult};
pub use submission::SubmissionOutcome;

pub use db::cook_log;
pub use routes::router;
pub use stored::StoredEntry;
