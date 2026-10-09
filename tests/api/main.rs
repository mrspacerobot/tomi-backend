//! End-to-end tests of the API against a real Postgres, as one test binary.
//!
//! `sqlx::test` creates a fresh database per test from `DATABASE_URL` and
//! runs the migrations; see `scripts/check.sh`. One file per feature.

#![expect(clippy::unwrap_used, reason = "a failed unwrap in a test is a failed test, not a crash")]

mod auth;
mod client;
mod consents;
mod fakes;
mod health;
mod helpers;
mod profile;
mod scans;
mod specials;
