use super::Catalog;
use std::collections::BTreeMap;

/// Each skill id with its level numbers, in catalog order.
type SkillLevels = BTreeMap<String, Vec<u8>>;

/// Every skill and level ever released. Cook entries keep these forever.
fn released() -> SkillLevels {
    serde_json::from_str(include_str!("../../../library/catalog.released.json")).unwrap()
}

fn shipped() -> Catalog {
    Catalog::parse(include_str!("../../../library/catalog.json")).unwrap()
}

fn levels_of(catalog: &Catalog) -> SkillLevels {
    let numbers = |skill: &super::Skill| skill.levels.iter().map(|level| level.number).collect();
    catalog.skills.iter().map(|skill| (skill.id.to_string(), numbers(skill))).collect()
}

/// The released skills and levels a catalog no longer has under the same id and number.
fn lost_from_release(catalog: &Catalog) -> Vec<String> {
    let now = levels_of(catalog);
    let mut lost = Vec::new();
    for (skill, numbers) in released() {
        let Some(found) = now.get(&skill) else {
            lost.push(format!("skill `{skill}`"));
            continue;
        };
        let missing = numbers.iter().filter(|number| !found.contains(number));
        lost.extend(missing.map(|number| format!("`{skill}` level {number}")));
    }
    lost
}

#[test]
fn the_catalog_is_exactly_the_released_one() {
    // A new skill or level is a change cook entries keep forever. Add it to `library/catalog.released.json` in
    // the same pull request so a person sees it.
    assert_eq!(levels_of(&shipped()), released());
}

#[test]
fn nothing_released_is_missing_from_the_shipped_catalog() {
    assert_eq!(lost_from_release(&shipped()), Vec::<String>::new());
}

#[test]
fn removing_a_released_skill_is_caught() {
    let mut catalog = shipped();
    let removed = catalog.skills.pop().unwrap();
    assert_eq!(lost_from_release(&catalog), vec![format!("skill `{}`", removed.id)]);
}

#[test]
fn removing_a_released_level_is_caught() {
    let mut catalog = shipped();
    let removed = catalog.skills[0].levels.pop().unwrap();
    let expected = format!("`{}` level {}", catalog.skills[0].id, removed.number);
    assert_eq!(lost_from_release(&catalog), vec![expected]);
}

#[test]
fn renumbering_a_released_level_is_caught() {
    let mut catalog = shipped();
    let level = &mut catalog.skills[0].levels[0];
    let old_number = level.number;
    level.number = u8::MAX;
    let expected = format!("`{}` level {old_number}", catalog.skills[0].id);
    assert_eq!(lost_from_release(&catalog), vec![expected]);
}
