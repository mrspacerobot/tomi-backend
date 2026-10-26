//! The levels suggestions may offer, on the shipped catalog.

use super::test_log::{cooks, id, TestLog};
use crate::library::tests::shipped_files;
use crate::library::{Catalog, Library};
use crate::profile::Experience;
use crate::progress::{open_levels, CookCounts, OpenLevels};

fn catalog() -> Catalog {
    Library::parse(&shipped_files()).unwrap().catalog
}

fn open(experience: Experience, counts: &CookCounts) -> OpenLevels {
    open_levels(&catalog(), experience, counts)
}

fn knife_cuts(count: u32) -> CookCounts {
    CookCounts::from_log(&cooks(count, "knife-cuts", 1))
}

#[test]
fn a_new_cook_starts_at_level_one_of_skills_without_prerequisites() {
    let open = open(Experience::Basics, &CookCounts::default());
    assert!(open.contains(&id("knife-cuts"), 1));
    assert!(open.contains(&id("eggs"), 1));
    assert!(!open.contains(&id("eggs"), 2));
    assert!(!open.contains(&id("saute"), 1), "saute needs knife cuts first");
}

#[test]
fn comfortable_counts_the_first_course_as_complete() {
    let open = open(Experience::Comfortable, &CookCounts::default());
    assert!(open.contains(&id("eggs"), 3));
    assert!(open.contains(&id("saute"), 1));
    assert!(!open.contains(&id("saute"), 2));
    assert!(!open.contains(&id("sear"), 1), "sear needs saute level 1 cooked twice");
}

#[test]
fn two_cooks_complete_a_level_and_open_what_follows() {
    let open = open(Experience::Never, &knife_cuts(2));
    assert!(open.contains(&id("knife-cuts"), 2));
    assert!(open.contains(&id("saute"), 1));
    assert!(open.contains(&id("stocks"), 1));
}

#[test]
fn one_cook_is_not_enough() {
    assert!(!open(Experience::Never, &knife_cuts(1)).contains(&id("knife-cuts"), 2));
}

#[test]
fn a_mastered_skill_without_its_prerequisites_opens_no_level() {
    let log = TestLog::new();
    let roux = CookCounts::from_log(&[cooks(2, "roux", 1), cooks(2, "roux", 2)].concat());
    let open = open_levels(&log.catalog, Experience::Basics, &roux);
    assert!(!open.contains(&id("roux"), 1));
    assert!(open.contains(&id("knife"), 1));
}
