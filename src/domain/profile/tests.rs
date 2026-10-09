use super::*;
use serde_json::json;

fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

fn patch(value: serde_json::Value) -> ProfilePatch {
    serde_json::from_value(value).unwrap()
}

fn vegetarian_without_nuts() -> Profile {
    Profile { diet: Some(id("vegetarian")), avoid: [id("nuts")].into(), ..Profile::default() }
}

#[test]
fn adding_a_restriction_is_allowed() {
    let start = vegetarian_without_nuts();
    let next = apply(&start, patch(json!({"version": 1, "avoid": ["nuts", "sesame"]}))).unwrap();
    assert!(next.avoid.contains(&id("sesame")));
}

#[test]
fn dropping_an_avoid_item_silently_is_refused() {
    let result = apply(&vegetarian_without_nuts(), patch(json!({"version": 1, "avoid": []})));
    let removed = vec!["avoid: nuts".to_string()];
    assert_eq!(result, Err(PatchError::WeakensRestrictions { removed }));
}

#[test]
fn clearing_or_changing_the_diet_silently_is_refused() {
    for diet in [json!(null), json!("pescatarian")] {
        let result = apply(&vegetarian_without_nuts(), patch(json!({"version": 1, "diet": diet})));
        let removed = vec!["diet: vegetarian".to_string()];
        assert_eq!(result, Err(PatchError::WeakensRestrictions { removed }));
    }
}

#[test]
fn dropping_a_restriction_on_request_is_allowed() {
    let body = json!({"version": 1, "avoid": [], "diet": null, "allowWeakerRestrictions": true});
    let next = apply(&vegetarian_without_nuts(), patch(body)).unwrap();
    assert!(next.avoid.is_empty());
    assert_eq!(next.diet, None);
}

#[test]
fn a_patch_without_restrictions_leaves_them_alone() {
    let start = vegetarian_without_nuts();
    let next = apply(&start, patch(json!({"version": 1, "servings": 4}))).unwrap();
    assert_eq!(next.avoid, start.avoid);
    assert_eq!(next.diet, start.diet);
    assert_eq!(next.servings, 4);
}

#[test]
fn null_clears_a_field_and_absence_keeps_it() {
    let start = Profile { equipment: Some([id("wok")].into()), ..Profile::default() };
    let cleared = apply(&start, patch(json!({"version": 1, "equipment": null}))).unwrap();
    assert_eq!(cleared.equipment, None);
    let kept = apply(&start, patch(json!({"version": 1}))).unwrap();
    assert_eq!(kept.equipment, start.equipment);
}

#[test]
fn a_cuisine_with_two_answers_is_refused() {
    let body = json!({"version": 0, "cuisinesLoved": ["thai"], "cuisinesNever": ["thai"]});
    let result = apply(&Profile::default(), patch(body));
    let problems = vec!["cuisine `thai` is in more than one list".to_string()];
    assert_eq!(result, Err(PatchError::Invalid(problems)));
}

#[test]
fn numbers_out_of_range_are_refused() {
    let body = json!({"version": 0, "servings": 0, "weeklyGoal": 8});
    let Err(PatchError::Invalid(problems)) = apply(&Profile::default(), patch(body)) else {
        panic!("expected the patch to be refused");
    };
    assert_eq!(problems.len(), 2);
}

#[test]
fn unknown_fields_and_values_are_refused() {
    let bodies = [
        json!({"version": 1, "xp": 9000}),
        json!({"version": 1, "experience": "expert"}),
        json!({"version": 1, "units": "imperial"}),
        json!({"version": 1, "avoid": ["Nuts"]}),
    ];
    for body in bodies {
        let result: Result<ProfilePatch, _> = serde_json::from_value(body.clone());
        assert!(result.is_err(), "{body}");
    }
}
