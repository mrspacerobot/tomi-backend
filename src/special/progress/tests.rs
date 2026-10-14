use super::*;
use crate::library::tests::shipped_files;
use crate::library::Library;

fn catalog() -> Catalog {
    Library::parse(&shipped_files()).unwrap().0.catalog
}

fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

fn open(experience: Experience, counts: &CookCounts) -> OpenLevels {
    open_levels(&catalog(), experience, counts)
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
    let counts = CookCounts(BTreeMap::from([((id("knife-cuts"), 1), 2)]));
    let open = open(Experience::Never, &counts);
    assert!(open.contains(&id("knife-cuts"), 2));
    assert!(open.contains(&id("saute"), 1));
    assert!(open.contains(&id("stocks"), 1));
}

#[test]
fn one_cook_is_not_enough() {
    let counts = CookCounts(BTreeMap::from([((id("knife-cuts"), 1), 1)]));
    assert!(!open(Experience::Never, &counts).contains(&id("knife-cuts"), 2));
}
