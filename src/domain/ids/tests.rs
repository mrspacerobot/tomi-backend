use super::*;

#[test]
fn accepts_ids_of_tomis_library() {
    for raw in ["nuts", "vegetarian", "olive-oil", "black-pepper", "knife-1"] {
        assert!(SavedId::parse(raw).is_ok(), "{raw}");
    }
}

#[test]
fn refuses_ids_of_the_wrong_shape() {
    let refused = ["", " ", "Nuts", "olive oil", "olive--oil", "-nuts", "nuts-", "fact/nuts", "nüsse"];
    for raw in refused {
        assert!(SavedId::parse(raw).is_err(), "{raw:?}");
    }
}

#[test]
fn refuses_ids_that_are_too_long() {
    let long = "a".repeat(MAX_ID_LENGTH + 1);
    assert!(SavedId::parse(&long).is_err());
}
