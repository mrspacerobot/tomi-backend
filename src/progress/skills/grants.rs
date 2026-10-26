//! What the cook's experience answer is worth: "comfortable" grants the first course.

use crate::kernel::ids::SavedId;
use crate::library::Catalog;
use crate::profile::Experience;
use std::collections::BTreeSet;

/// Every skill of the first course in the catalog, for a cook who answered "comfortable".
/// "Never" and "basics" differ only in how much Tomi explains, so neither grants anything.
pub fn granted_skills(catalog: &Catalog, experience: Experience) -> BTreeSet<SavedId> {
    if experience != Experience::Comfortable {
        return BTreeSet::new();
    }
    let Some(first_course) = catalog.courses.first() else {
        return BTreeSet::new();
    };
    let in_course = catalog.skills.iter().filter(|skill| skill.course == first_course.id);
    in_course.map(|skill| skill.id.clone()).collect()
}
