//! Rules that decide without a network, a database or a clock.
//!
//! Everything here is plain data in and plain data out, so every rule can
//! be tested on its own. `tests/architecture.rs` fails when a file in this
//! folder reaches for the web framework, the database or the current time.

pub mod consent;
pub mod display;
pub mod fridge;
pub mod game;
pub mod graph;
pub mod ids;
pub mod language;
pub mod library;
pub mod options;
pub mod photo;
pub mod profile;
pub mod progress;
pub mod resolve;
pub mod special;
