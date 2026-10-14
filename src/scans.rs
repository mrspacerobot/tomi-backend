//! The fridge scan: a photo in, what the model saw out, and keeping the photo.
//!
//! `fridge` and `photo` are pure rules. `routes` is the HTTP handler, `db` the
//! queries and `keep` the storage step.

pub mod db;
pub mod fridge;
mod keep;
pub mod photo;
mod routes;

pub use routes::router;
