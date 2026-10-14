use super::*;

#[test]
fn reads_the_first_language_the_app_asks_for() {
    assert_eq!(Language::from_accept_language(Some("de-CH,de;q=0.9,en;q=0.8")).as_str(), "de");
    assert_eq!(Language::from_accept_language(Some("FR")).as_str(), "fr");
}

#[test]
fn falls_back_to_english() {
    for header in [None, Some(""), Some("*"), Some("12"), Some("x-klingon")] {
        assert_eq!(Language::from_accept_language(header), Language::english(), "{header:?}");
    }
}
