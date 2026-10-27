//! The server-run quiz: each pick is checked against the pinned graph, and each question's first answer is kept.
//!
//! Correct answers never leave this server. A pick is answered right or wrong, with the graph's explanation only
//! once it is right. The first answer to each question in one cook is stored and is the score; later picks of the
//! same question only teach. Answers belong to one Supabase account and meet their cook entry by its id.

mod db;
mod pick;
mod recorded;
mod routes;
#[cfg(test)]
mod tests;

pub use db::first_answers;
pub use recorded::RecordedAnswer;
pub use routes::router;
