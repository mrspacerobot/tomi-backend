use super::*;
use crate::kernel::ids::SavedId;
use std::collections::BTreeSet;

pub(crate) fn shipped_files() -> LibraryFiles {
    LibraryFiles {
        catalog: include_str!("../../library/catalog.json").to_string(),
        restrictions: include_str!("../../library/restrictions.json").to_string(),
        staples: include_str!("../../library/staples.json").to_string(),
        equipment: include_str!("../../library/equipment.json").to_string(),
        choices: include_str!("../../library/choices.json").to_string(),
    }
}

fn shipped() -> Library {
    Library::parse(&shipped_files()).unwrap()
}

fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

fn avoided(diet: Option<&str>, avoid: &[&str]) -> Result<BTreeSet<String>, Refusal> {
    let diet = diet.map(id);
    let avoid: BTreeSet<SavedId> = avoid.iter().map(|raw| id(raw)).collect();
    let cook = Restrictions { diet: diet.as_ref(), avoid: &avoid };
    shipped().restrictions.facts_to_avoid(&cook)
}

fn facts(raw: &[&str]) -> BTreeSet<String> {
    raw.iter().map(|fact| format!("fact/{fact}")).collect()
}

#[test]
fn the_shipped_library_parses_whole() {
    let library = Library::parse(&shipped_files()).unwrap();
    assert_eq!(library.restrictions.diets.len(), 3);
}

#[test]
fn diets_and_avoid_items_stand_for_the_graph_facts_the_owner_mapped() {
    assert_eq!(
        avoided(Some("vegetarian"), &[]),
        Ok(facts(&["crustaceans", "fish", "meat", "molluscs", "pork", "slaughter-product"]))
    );
    assert_eq!(avoided(None, &["dairy"]), Ok(facts(&["milk"])));
    assert_eq!(avoided(None, &["nuts"]), Ok(facts(&["peanuts", "tree-nuts"])));
    assert!(avoided(Some("vegan"), &["gluten"]).unwrap().contains("fact/wheat"));
}

#[test]
fn a_diet_or_avoid_item_the_list_does_not_have_refuses() {
    assert_eq!(avoided(Some("fruitarian"), &[]), Err(Refusal::UnknownDiet(id("fruitarian"))));
    assert_eq!(avoided(None, &["mango"]), Err(Refusal::UnknownAvoidItem(id("mango"))));
}

#[test]
fn the_shipped_restrictions_name_only_facts_the_graph_has() {
    let graph = crate::graph::test_helpers::fixture_graph();
    assert_eq!(shipped().restrictions.check_against(&graph.serving.facts), Ok(()));
}

#[test]
fn a_broken_restrictions_file_rejects_the_library() {
    let text = shipped_files().restrictions;
    let broken = [
        text.replacen("\"fact/milk\"", "\"milk\"", 1),
        text.replacen("\"schemaVersion\": 2", "\"schemaVersion\": 3", 1),
        text.replacen("\"avoids\": [", "\"tags\": [], \"avoids\": [", 1),
    ];
    for broken_text in broken {
        assert_ne!(broken_text, text, "the test edit did not apply");
        let files = LibraryFiles { restrictions: broken_text, ..shipped_files() };
        assert!(Library::parse(&files).is_err());
    }
}

/// Every avoid item the library ends up with, by id, with the facts it rules out.
fn released() -> std::collections::BTreeMap<String, BTreeSet<String>> {
    let text = include_str!("../../library/avoid-items.released.json");
    serde_json::from_str(text).unwrap()
}

fn current(library: &Library) -> std::collections::BTreeMap<String, BTreeSet<String>> {
    let items = library.restrictions.item_facts();
    items.iter().map(|(id, facts)| (id.to_string(), facts.clone())).collect()
}

fn with_graph() -> Library {
    let mut library = shipped();
    library.agree_with(&crate::graph::test_helpers::fixture_graph()).unwrap();
    library
}

#[test]
fn the_avoid_items_are_exactly_the_released_ones() {
    // A change to this list is a change to what a cook with an allergy is protected from. Edit
    // `library/avoid-items.released.json` in the same pull request so a person sees it.
    assert_eq!(current(&with_graph()), released());
}

#[test]
fn a_released_avoid_item_is_never_removed_and_never_rules_out_less() {
    let now = current(&with_graph());
    for (id, facts) in released() {
        let found = now.get(&id).unwrap_or_else(|| panic!("`{id}` was released and is gone"));
        assert!(found.is_superset(&facts), "`{id}` rules out less than it did");
    }
}
