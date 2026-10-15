//! Reading a stored profile row by the schema version it was saved at.
//!
//! To add a profile field: give it a default in `Profile` (one that never weakens a
//! restriction), raise `PROFILE_SCHEMA_VERSION`, and say here how rows of the older
//! versions become the new shape. Usually that is nothing: the default fills the field.
//!
//! Version 2 added `cupboardAnswered`. A row from version 1 that already has staples saved was
//! answered, so it reads as answered; one with none reads as not answered.
//! A server never reads a row from a newer version than its own. The row could hold a
//! restriction this server does not know, and leaving it out would weaken a filter.

use super::{Profile, PROFILE_SCHEMA_VERSION};
use serde_json::Value;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum StoredProfileError {
    #[error("the profile was saved by a newer server, at schema version {0}")]
    NewerThanThisServer(i32),
    #[error("the stored profile could not be read")]
    Unreadable,
}

pub fn from_stored(schema_version: i32, data: Value) -> Result<Profile, StoredProfileError> {
    if schema_version > PROFILE_SCHEMA_VERSION {
        return Err(StoredProfileError::NewerThanThisServer(schema_version));
    }
    if schema_version < 1 {
        return Err(StoredProfileError::Unreadable);
    }
    let mut profile: Profile = serde_json::from_value(data).map_err(|_| StoredProfileError::Unreadable)?;
    if schema_version < 2 && !profile.cupboard_staples.is_empty() {
        profile.cupboard_answered = true;
    }
    Ok(profile)
}
