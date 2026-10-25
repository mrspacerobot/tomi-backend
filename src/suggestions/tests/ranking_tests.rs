use super::*;

#[test]
fn overlap_order_puts_fewest_missing_first() {
    let at_home = ["egg", "spinach", "vegetarian-hard-cheese", "butter", "salt", "black-pepper"];
    let order = overlap_order(offered(&Profile::default(), &at_home));
    assert_eq!(candidate_ids(&order), vec![OMELETTE, "recipe/soft-scrambled-eggs", PAN_SAUCE]);
}

#[test]
fn confirmed_food_overlap_beats_assumed_pantry_overlap() {
    let pantry_foods = ["egg", "spinach", "vegetarian-hard-cheese", "butter", "salt", "black-pepper"];
    let mut candidates = offered(&Profile::default(), &pantry_foods);
    for candidate in &mut candidates {
        candidate.confirmed_ingredients.clear();
    }
    let omelette = candidates.iter_mut().find(|candidate| candidate.recipe_id == OMELETTE).unwrap();
    omelette.confirmed_ingredients.push("ingredient/egg".into());
    assert_eq!(overlap_order(candidates)[0].recipe_id, OMELETTE);
}
