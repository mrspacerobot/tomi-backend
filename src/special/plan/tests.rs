use super::*;
use crate::graph::serving::ServedRecipe;

#[test]
fn preparation_ids_survive_the_graph_safe_plan() {
    let recipe: ServedRecipe = serde_json::from_value(recipe_with_preparation()).unwrap();
    let found = Planned::read(&recipe, &RecipePlan::default());
    assert_eq!(found.ingredients, vec!["preparation/garam-masala"]);
}

fn recipe_with_preparation() -> Value {
    serde_json::json!({
        "makes": "dish/test", "tier": "drafted", "techniques": [],
        "ingredients": ["preparation/garam-masala"], "equipment": [],
        "unsafe_for": [], "optional_parts": [],
        "lines": [{"thing": "preparation/garam-masala", "part": "garam-masala",
            "optional": false, "unsafe_for": [], "swaps": []}]
    })
}
