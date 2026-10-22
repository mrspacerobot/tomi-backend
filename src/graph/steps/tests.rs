use super::*;
use std::collections::BTreeSet;

fn written(text: &str, left_out: &[&str], name: &str) -> Result<String, StepError> {
    let left_out: BTreeSet<String> = left_out.iter().map(|part| part.to_string()).collect();
    let proper = vec!["Dijon".to_string()];
    let context = RenderContext { left_out: &left_out, proper_name_words: &proper };
    render(&parse(text)?, &context, |_| Ok(name.to_string()))
}

#[test]
fn a_placeholder_is_written_as_the_name_in_use() {
    assert_eq!(written("Melt the {butter} slowly.", &[], "Olive oil").unwrap(), "Melt the olive oil slowly.");
}

#[test]
fn a_name_starting_a_sentence_gets_a_capital() {
    assert_eq!(written("{egg} first. Then {egg}.", &[], "egg").unwrap(), "Egg first. Then egg.");
}

#[test]
fn a_proper_name_keeps_its_capital_inside_a_sentence() {
    assert_eq!(written("Stir in the {mustard}.", &[], "Dijon mustard").unwrap(), "Stir in the Dijon mustard.");
}

#[test]
fn a_or_an_follows_the_name_in_use() {
    assert_eq!(written("Peel an {onion} and a {onion}.", &[], "shallot").unwrap(), "Peel a shallot and a shallot.");
    assert_eq!(written("Peel a {onion}. An {onion}.", &[], "egg").unwrap(), "Peel an egg. An egg.");
    assert_eq!(written("A man {x} and a {x}", &[], "apple").unwrap(), "A man apple and an apple");
}

#[test]
fn a_kept_clause_loses_only_its_brackets() {
    let text = "Stir in the {butter} [cream: and the {cream}] to stop.";
    assert_eq!(written(text, &[], "cream").unwrap(), "Stir in the cream and the cream to stop.");
}

#[test]
fn a_left_out_clause_takes_the_space_before_it() {
    let text = "Stir in the {butter} [cream: and the {cream}] to stop.";
    assert_eq!(written(text, &["cream"], "butter").unwrap(), "Stir in the butter to stop.");
}

#[test]
fn a_step_that_is_only_a_left_out_clause_is_empty() {
    assert_eq!(written("[egg: Peel the {egg}.]", &["egg"], "egg").unwrap(), "");
}

#[test]
fn the_name_resolver_decides_plural_and_unknown_parts() {
    let pieces = parse("the {egg:plural}").unwrap();
    assert_eq!(pieces[1], Piece::Mention(Mention { part: "egg".into(), ingredient: None, plural: true }));
    let context = RenderContext { left_out: &BTreeSet::new(), proper_name_words: &[] };
    let refused = render(&pieces, &context, |mention| Err(StepError::UnknownPlaceholder(mention.part.clone())));
    assert_eq!(refused, Err(StepError::UnknownPlaceholder("egg".into())));
}

#[test]
fn text_that_is_not_in_the_format_is_refused() {
    for text in
        ["Melt the {butter", "a stray ] here", "a } stray", "[a: x [b: y]]", "[a: never closed", "{Bad}", "{a.b.c}"]
    {
        assert!(matches!(parse(text), Err(StepError::Malformed(_))), "{text}");
    }
}

#[test]
fn an_ingredient_slug_is_read() {
    let pieces = parse("{roux.butter:plural}").unwrap();
    assert_eq!(
        pieces,
        vec![Piece::Mention(Mention { part: "roux".into(), ingredient: Some("butter".into()), plural: true })]
    );
}
