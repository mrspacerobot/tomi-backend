use super::*;
use std::collections::BTreeSet;

pub(crate) fn shipped_files() -> LibraryFiles {
    let lesson = |name: &str, text: &str| (name.to_string(), text.to_string());
    LibraryFiles {
        catalog: include_str!("../../library/catalog.json").to_string(),
        restrictions: include_str!("../../library/restrictions.json").to_string(),
        equipment: include_str!("../../library/equipment.json").to_string(),
        choices: include_str!("../../library/choices.json").to_string(),
        lessons: vec![
            lesson("omelette", include_str!("../../library/lessons/spinach-hard-cheese-omelette.lesson.json")),
            lesson(
                "pan-sauce",
                include_str!("../../library/lessons/seared-chicken-lemon-butter-pan-sauce.lesson.json"),
            ),
        ],
    }
}

fn shipped() -> Library {
    Library::parse(&shipped_files()).unwrap().0
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
    let (library, left_out) = Library::parse(&shipped_files()).unwrap();
    assert!(left_out.is_empty(), "{left_out:?}");
    assert_eq!(library.lessons.len(), 2);
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
    let graph = crate::special::links::tests::fixture_graph();
    assert_eq!(shipped().restrictions.check_against(&graph.serving.facts), Ok(()));
}

#[test]
fn a_broken_restrictions_file_rejects_the_library() {
    let text = shipped_files().restrictions;
    let broken = [
        text.replacen("\"fact/milk\"", "\"milk\"", 1),
        text.replacen("\"schemaVersion\": 1", "\"schemaVersion\": 2", 1),
        text.replacen("\"avoids\": [", "\"tags\": [], \"avoids\": [", 1),
    ];
    for broken_text in broken {
        assert_ne!(broken_text, text, "the test edit did not apply");
        let files = LibraryFiles { restrictions: broken_text, ..shipped_files() };
        assert!(Library::parse(&files).is_err());
    }
}

#[test]
fn a_lesson_at_a_level_the_catalog_lacks_is_left_out() {
    let mut files = shipped_files();
    files.lessons[0].1 = files.lessons[0].1.replacen("\"level\": 3", "\"level\": 9", 1);
    let (_, left_out) = Library::parse(&files).unwrap();
    assert!(left_out[0].contains("has no level 9"), "{left_out:?}");
}
