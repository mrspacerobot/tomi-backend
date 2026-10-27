//! What time it is, read in one place so a test can fix it.
//!
//! Routes ask the clock once per request and pass "now" into the rules, which never read a clock themselves.

use jiff::Timestamp;

/// The moment a request is answered at.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// The server's own clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now()
    }
}
