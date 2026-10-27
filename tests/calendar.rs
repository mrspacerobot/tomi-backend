//! Calendar validation is shared by request and cook-entry handling.

use tomi_api::kernel::calendar::known_zone;

#[test]
fn named_zones_are_accepted_and_fixed_offsets_and_unknown_zones_are_refused() {
    assert!(known_zone("Europe/Amsterdam").is_some());
    assert!(known_zone("UTC").is_some());
    assert!(known_zone("+02:00").is_none());
    assert!(known_zone("Mars/Olympus_Mons").is_none());
}
