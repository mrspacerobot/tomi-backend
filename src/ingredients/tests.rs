use super::*;
use crate::kernel::language::Language;
use crate::library::tests::shipped_files;
use crate::library::Library;

fn resolved(label: &str) -> Resolved {
    let library = Library::parse(&shipped_files()).unwrap().0;
    let language = Language::english();
    resolve(label, &Namer { library: &library, language: &language })
}

fn candidate_ids(found: &Resolved) -> Vec<String> {
    found.candidates.iter().map(|named| named.id.to_string()).collect()
}

#[test]
fn a_name_in_any_case_or_plural_is_exact() {
    for label in ["Egg", "  eggs ", "OLIVE OIL", "olive-oil", "Tinned tomatoes", "tinned tomato"] {
        let found = resolved(label);
        assert_eq!(found.basis, Basis::Exact, "{label}");
        assert_eq!(found.candidates.len(), 1, "{label}");
    }
}

#[test]
fn a_word_inside_names_is_offered_to_choose_from() {
    let found = resolved("stock");
    assert_eq!(found.basis, Basis::Exact, "`stock` is an ingredient of its own");
    let found = resolved("cheese");
    assert_eq!(found.basis, Basis::Ambiguous);
    assert_eq!(candidate_ids(&found), vec!["vegetarian-hard-cheese"]);
    let found = resolved("oil");
    assert_eq!(found.basis, Basis::Ambiguous);
    assert_eq!(candidate_ids(&found), vec!["olive-oil", "sunflower-oil"]);
}

#[test]
fn nothing_known_is_none() {
    let found = resolved("dragon fruit");
    assert_eq!(found.basis, Basis::None);
    assert!(found.candidates.is_empty());
}
