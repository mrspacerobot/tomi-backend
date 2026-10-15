use super::*;
use crate::special::links::tests::fixture_graph;

fn item(label: &str, ingredient: Option<&str>, confidence: f64) -> ModelItem {
    ModelItem { label: label.into(), ingredient: ingredient.map(Into::into), confidence }
}

fn foods() -> Foods {
    fixture_graph().foods
}

#[test]
fn known_ingredients_are_kept_and_others_become_plain_labels() {
    let items = vec![
        item("eggs", Some("egg"), 0.9),
        item("jar of pesto", Some("pesto"), 0.7),
        item("leeks", Some("Leek!"), 0.5),
    ];
    let seen = checked(items, &foods());
    assert_eq!(seen[0].ingredient.as_ref().map(|id| id.as_str()), Some("egg"));
    assert_eq!(seen[1].ingredient, None);
    assert_eq!(seen[2].ingredient, None);
}

#[test]
fn confidence_is_kept_between_zero_and_one() {
    let items = vec![item("a", None, 7.0), item("b", None, -1.0), item("c", None, f64::NAN)];
    let confidences: Vec<f64> = checked(items, &foods()).iter().map(|seen| seen.confidence).collect();
    assert_eq!(confidences, vec![1.0, 0.0, 0.0]);
}

#[test]
fn blank_labels_are_dropped_and_long_answers_cut() {
    let mut items = vec![item("   ", Some("egg"), 1.0)];
    items.extend((0..100).map(|number| item(&"x".repeat(number + 200), None, 0.5)));
    let seen = checked(items, &foods());
    assert_eq!(seen.len(), MAX_ITEMS - 1);
    assert!(seen.iter().all(|item| item.label.chars().count() == 80));
}
