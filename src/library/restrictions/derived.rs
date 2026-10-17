//! Avoid items that come from the graph: every allergen fact no person-written item rules out gets one of its own.
//!
//! A person still writes the groupings (nuts, shellfish, gluten) and the choices (pork, alcohol). The graph only
//! fills gaps, so an allergen it knows can never be missing from the list. What it adds is listed in
//! `library/avoid-items.released.json`, which a test holds to, so a graph update cannot change it unseen.

use super::{Choice, Kind, RestrictionList, FACT_PREFIX};
use crate::graph::facts::{Concern, Facts};
use crate::kernel::ids::SavedId;
use std::collections::BTreeSet;

impl RestrictionList {
    /// Checks the person-written items against the graph's facts and adds the items the graph derives.
    pub fn extend_from_graph(&mut self, facts: &Facts) -> Result<(), String> {
        self.check_names_against(facts)?;
        self.kinds_from(facts);
        if self.allergens_from_graph {
            self.add_uncovered_allergens(facts)?;
        }
        self.known_facts = facts.ids().cloned().collect();
        Ok(())
    }

    /// An item named like a fact must rule that fact out, so a saved id never means less than its name says.
    fn check_names_against(&self, facts: &Facts) -> Result<(), String> {
        for (id, ruled_out) in &self.item_facts {
            let named = format!("{FACT_PREFIX}{id}");
            if facts.get(&named).is_some() && !ruled_out.contains(&named) {
                return Err(format!("`{id}` is named like the fact `{named}` but does not rule it out"));
            }
        }
        Ok(())
    }

    fn kinds_from(&mut self, facts: &Facts) {
        for (id, ruled_out) in &self.item_facts {
            let allergen = ruled_out.iter().any(|fact| is_allergen(facts, fact));
            self.kinds.insert(id.clone(), if allergen { Kind::Allergen } else { Kind::Choice });
        }
    }

    fn add_uncovered_allergens(&mut self, facts: &Facts) -> Result<(), String> {
        let covered: BTreeSet<String> = self.item_facts.values().flatten().cloned().collect();
        for (fact_id, info) in facts.allergens().filter(|(fact_id, _)| !covered.contains(*fact_id)) {
            let slug = fact_id.trim_start_matches(FACT_PREFIX);
            let id = SavedId::parse(slug).map_err(|_| format!("`{fact_id}` cannot be an avoid item id"))?;
            if self.item_facts.contains_key(&id) {
                return Err(format!("the derived avoid item `{id}` clashes with a written one"));
            }
            self.avoid_items.push(Choice::named(&id, &info.name));
            self.item_facts.insert(id.clone(), BTreeSet::from([fact_id.clone()]));
            self.kinds.insert(id, Kind::Allergen);
        }
        Ok(())
    }
}

fn is_allergen(facts: &Facts, fact: &str) -> bool {
    match facts.get(fact) {
        Some(info) => info.concern == Concern::Allergen,
        None => true,
    }
}
