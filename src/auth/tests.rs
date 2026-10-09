use super::*;

#[test]
fn reads_a_bearer_header() {
    assert_eq!(bearer(Some("Bearer abc")), Ok("abc"));
}

#[test]
fn refuses_other_header_values() {
    for value in [Some("Basic abc"), Some("Bearer "), Some("bearer abc"), None] {
        assert_eq!(bearer(value), Err(AuthError::Missing), "{value:?}");
    }
}
