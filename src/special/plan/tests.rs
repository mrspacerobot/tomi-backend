use super::*;
use crate::special::adapted::{AdaptedLine, Amount, Unit};

#[test]
fn preparation_ids_survive_the_adapted_plan() {
    let line = AdaptedLine {
        part: "spice".into(),
        food: "preparation/garam-masala".into(),
        amount: Amount { value: 1.0, unit: Unit::Teaspoon },
        forms: Vec::new(),
    };
    let adapted = Adapted {
        yields: Amount { value: 2.0, unit: Unit::Serving },
        lines: vec![line],
        steps: Vec::new(),
        swaps: Vec::new(),
        left_out: Vec::new(),
    };
    assert_eq!(food_ids(&adapted), vec!["preparation/garam-masala"]);
}
