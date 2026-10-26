//! End-to-end tests of the API against a real Postgres, as one test binary.
//!
//! `sqlx::test` creates a fresh database per test from `DATABASE_URL` and
//! runs the migrations; see `scripts/check.sh`. One file per feature.

#![expect(clippy::unwrap_used, reason = "a failed unwrap in a test is a failed test, not a crash")]

mod adapted;
mod ai_logs;
mod auth;
mod client;
mod cupboard;
mod fakes;
mod graph;
mod health;
mod helpers;
mod ingredients;
mod logs;
mod options;
mod profile;
mod progress;
mod recipes;
mod scans;
mod suggestions;
mod teaching;
