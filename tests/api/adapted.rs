use crate::fakes::FakeAi;
use crate::helpers::*;
use crate::suggestions::{comfortable_cook, open_server};
use serde_json::{json, Value};
use sqlx::PgPool;

const PAN_SAUCE: &str = "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce";
const EGGS: &str = "recipe/soft-scrambled-eggs";

fn offer_of<'a>(body: &'a Value, recipe: &str) -> &'a Value {
    let found = std::iter::once(&body["special"])
        .chain(body["alternatives"].as_array().unwrap())
        .find(|offer| offer["recipeId"] == recipe);
    found.unwrap()
}

fn step_texts(offer: &Value) -> Vec<&str> {
    offer["recipe"]["steps"].as_array().unwrap().iter().map(|step| step["text"].as_str().unwrap()).collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn an_offer_carries_the_recipe_written_for_the_cook(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["egg", "cream"]})).await;
    let offer = offer_of(&reply.body, EGGS);
    assert_eq!(offer["recipe"]["yields"], json!({"value": 2, "unit": "serving"}));
    assert_eq!(offer["recipe"]["ingredients"][0]["id"], "ingredient/egg");
    assert_eq!(offer["recipe"]["ingredients"][0]["name"], "Egg");
    assert_eq!(offer["recipe"]["ingredients"][0]["amount"], json!({"value": 5, "unit": "piece"}));
    assert_eq!(offer["recipe"]["steps"][1]["time"], json!({"from": 4, "to": 6, "unit": "minute"}));
    assert_eq!(offer["recipe"]["steps"][2]["safetyPoints"], json!(["raw-or-lightly-cooked-egg"]));
    assert_eq!(offer["changes"], json!({"swaps": [], "leftOut": []}));
    assert!(offer["sourceRecipe"]["steps"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_swapped_food_is_written_into_the_steps_and_listed_as_a_change(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"avoid": ["alcohol"]})).await;
    let at_home = json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "sunflower-oil"]});
    let reply = suggest(&app, &cook, at_home).await;
    let offer = offer_of(&reply.body, PAN_SAUCE);
    assert!(step_texts(offer).iter().any(|text| text.starts_with("Pour in the chicken stock")));
    assert!(step_texts(offer).iter().all(|text| !text.contains("wine")));
    let swap = &offer["changes"]["swaps"][0];
    assert_eq!(swap["removed"]["id"], "ingredient/white-wine");
    assert_eq!(swap["added"]["name"], "Chicken stock");
    assert_eq!(swap["ratio"]["by"], "volume");
    assert!(swap["caveat"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stand_in_the_cook_has_replaces_the_food_they_lack_in_every_step(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["egg", "creme-fraiche"]})).await;
    let offer = offer_of(&reply.body, EGGS);
    assert_eq!(offer["recipe"]["ingredients"][2]["name"], "Crème fraîche");
    assert!(step_texts(offer)[2].contains("the cold butter and the crème fraîche to stop"));
    assert!(step_texts(offer).iter().all(|text| !text.contains("cream")));
    assert_eq!(offer["changes"]["swaps"][0]["removed"]["name"], "Cream");
}
