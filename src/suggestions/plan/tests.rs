use super::*;
use crate::recipe::{PreparedIngredient, RecipeAmount, Unit};

#[test]
fn preparation_ids_survive_the_adapted_plan() {
    let line = PreparedIngredient {
        part: "spice".into(),
        food: "preparation/garam-masala".into(),
        amount: RecipeAmount { value: 1.0, unit: Unit::Teaspoon },
        forms: Vec::new(),
    };
    let adapted = PreparedRecipe {
        yields: RecipeAmount { value: 2.0, unit: Unit::Serving },
        lines: vec![line],
        steps: Vec::new(),
        swaps: Vec::new(),
        left_out: Vec::new(),
    };
    assert_eq!(food_ids(&adapted), vec!["preparation/garam-masala"]);
}
