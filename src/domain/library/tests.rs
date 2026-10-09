use super::*;
use std::collections::BTreeSet;

pub(crate) fn shipped_files() -> LibraryFiles {
    let lesson = |name: &str, text: &str| (name.to_string(), text.to_string());
    LibraryFiles {
        catalog: include_str!("../../../library/catalog.json").to_string(),
        ingredients: include_str!("../../../library/ingredients.json").to_string(),
        equipment: include_str!("../../../library/equipment.json").to_string(),
        lessons: vec![
            lesson("omelette", include_str!("../../../library/lessons/spinach-hard-cheese-omelette.lesson.json")),
            lesson(
                "pan-sauce",
                include_str!("../../../library/lessons/seared-chicken-lemon-butter-pan-sauce.lesson.json"),
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

fn check(ingredient: &str, diet: Option<&str>, avoid: &[&str]) -> Result<(), Refusal> {
    let diet = diet.map(id);
    let avoid: BTreeSet<SavedId> = avoid.iter().map(|raw| id(raw)).collect();
    let cook = Restrictions { diet: diet.as_ref(), avoid: &avoid };
    shipped().table.check(&id(ingredient), &cook)
}

#[test]
fn the_shipped_library_parses_whole() {
    let (library, left_out) = Library::parse(&shipped_files()).unwrap();
    assert!(left_out.is_empty(), "{left_out:?}");
    assert_eq!(library.lessons.len(), 2);
}

#[test]
fn diets_and_avoid_items_follow_the_table() {
    assert!(check("chicken-thigh", Some("vegetarian"), &[]).is_err());
    assert!(check("butter", Some("vegan"), &[]).is_err());
    assert!(check("parmesan", None, &["dairy"]).is_err());
    assert!(check("egg", None, &["eggs"]).is_err());
    assert_eq!(check("spinach", Some("vegan"), &["nuts", "dairy"]), Ok(()));
}

#[test]
fn anything_the_table_does_not_list_refuses() {
    assert_eq!(check("unicorn", None, &[]), Err(Refusal::UnknownIngredient(id("unicorn"))));
    assert_eq!(check("salt", Some("fruitarian"), &[]), Err(Refusal::UnknownDiet(id("fruitarian"))));
    assert_eq!(check("salt", None, &["mango"]), Err(Refusal::UnknownAvoidItem(id("mango"))));
}

#[test]
fn a_broken_table_rejects_the_library() {
    let table = shipped_files().ingredients;
    let broken = [
        table.replacen("\"contains\": []", "\"contains\": [\"moonDust\"]", 1),
        table.replacen("\"triggeredBy\": [\"nuts\"]", "\"triggeredBy\": []", 1),
        table.replacen("\"schemaVersion\": 1", "\"schemaVersion\": 2", 1),
        table.replacen("\"contains\": []", "\"contains\": [], \"tags\": []", 1),
    ];
    for text in broken {
        assert_ne!(text, table, "the test edit did not apply");
        let files = LibraryFiles { ingredients: text, ..shipped_files() };
        assert!(Library::parse(&files).is_err());
    }
}

#[test]
fn a_lesson_naming_an_unknown_ingredient_is_left_out() {
    let mut files = shipped_files();
    files.lessons[0].1 = files.lessons[0].1.replacen("\"spinach\"", "\"kale\"", 1);
    let (library, left_out) = Library::parse(&files).unwrap();
    assert_eq!(library.lessons.len(), 1);
    assert!(left_out[0].contains("unknown ingredient `kale`"), "{left_out:?}");
}

#[test]
fn a_lesson_at_a_level_the_catalog_lacks_is_left_out() {
    let mut files = shipped_files();
    files.lessons[0].1 = files.lessons[0].1.replacen("\"level\": 3", "\"level\": 9", 1);
    let (_, left_out) = Library::parse(&files).unwrap();
    assert!(left_out[0].contains("has no level 9"), "{left_out:?}");
}
