//! Photo work against the pinned graph: missing dishes, changed recipes and orphans.

use super::{dish_source, fingerprint, Manifest, Record};
use crate::graph::Graph;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct Plan<'a> {
    missing: Vec<Dish<'a>>,
    stale: Vec<Dish<'a>>,
    orphaned: Vec<&'a str>,
    records: &'a BTreeMap<String, Record>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Dish<'a> {
    recipe: &'a str,
    source: &'a Value,
    recipe_fingerprint: String,
}

impl Manifest {
    pub fn plan<'a>(&'a self, graph: &'a Graph) -> Plan<'a> {
        let mut plan = Plan { records: &self.0, missing: Vec::new(), stale: Vec::new(), orphaned: Vec::new() };
        for recipe in graph.serving.recipes.keys() {
            let Some(source) = dish_source(graph, recipe) else {
                continue;
            };
            let dish = Dish { recipe, source, recipe_fingerprint: fingerprint(source) };
            match self.0.get(recipe) {
                None => plan.missing.push(dish),
                Some(record) if record.recipe_fingerprint != dish.recipe_fingerprint => plan.stale.push(dish),
                Some(_) => {}
            }
        }
        plan.orphaned =
            self.0.keys().filter(|recipe| dish_source(graph, recipe).is_none()).map(String::as_str).collect();
        plan
    }
}
