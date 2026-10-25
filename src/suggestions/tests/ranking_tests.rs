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

#[test]
fn a_model_may_only_reorder_the_list() {
    let by_overlap = overlap_order(offered(&Profile::default(), &[]));
    let model = [
        Ranked { recipe_id: "recipe/made-up".into(), reason: Some("invented".into()) },
        Ranked { recipe_id: OMELETTE.into(), reason: Some("quick tonight".into()) },
        Ranked { recipe_id: OMELETTE.into(), reason: None },
    ];
    let order = ranked(by_overlap, &model);
    let order_ids: Vec<String> = order.iter().map(|(candidate, _)| candidate.recipe_id.clone()).collect();
    assert_eq!(order_ids, vec![OMELETTE, "recipe/soft-scrambled-eggs", PAN_SAUCE]);
    assert_eq!(order[0].1.as_deref(), Some("quick tonight"));
}
