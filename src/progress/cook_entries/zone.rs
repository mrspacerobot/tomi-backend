//! IANA time zones, looked up in the time zone database bundled with `jiff`, never the machine's.

use jiff::tz::TimeZone;

/// The zone's canonical IANA name, or none when the database does not know it. Fixed offsets such as
/// `+02:00` are not zones: they would count a cook on the wrong day after a daylight saving change.
pub fn known_zone(name: &str) -> Option<String> {
    let Ok(zone) = TimeZone::get(name) else {
        return None;
    };
    let canonical = zone.iana_name()?;
    Some(canonical.to_string())
}
