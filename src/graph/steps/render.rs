//! Writes the text a cook reads from the pieces of a step, once each placeholder has the name of the food in use.

use super::{Clause, Mention, Piece, StepError};
use std::collections::BTreeSet;

const VOWELS: &str = "aeiouAEIOU";

/// What the words of a step depend on besides the foods.
pub struct RenderContext<'a> {
    /// Parts the cook goes without: their clauses are dropped.
    pub left_out: &'a BTreeSet<String>,
    /// Words that keep their capital inside a sentence, such as Dijon.
    pub proper_name_words: &'a [String],
}

/// The text, or an empty text when the whole step goes. `name_of` gives the English name of the food in use.
pub fn render<F>(pieces: &[Piece], context: &RenderContext, name_of: F) -> Result<String, StepError>
where
    F: Fn(&Mention) -> Result<String, StepError>,
{
    let mut writing = Writing { context, name_of, written: String::new() };
    writing.add_pieces(pieces)?;
    Ok(writing.written.trim().to_string())
}

struct Writing<'a, F> {
    context: &'a RenderContext<'a>,
    name_of: F,
    written: String,
}

impl<F: Fn(&Mention) -> Result<String, StepError>> Writing<'_, F> {
    fn add_pieces(&mut self, pieces: &[Piece]) -> Result<(), StepError> {
        for piece in pieces {
            match piece {
                Piece::Text(text) => self.written.push_str(text),
                Piece::Mention(mention) => self.add_mention(mention)?,
                Piece::Clause(clause) => self.add_clause(clause)?,
            }
        }
        Ok(())
    }

    fn add_mention(&mut self, mention: &Mention) -> Result<(), StepError> {
        let name = (self.name_of)(mention)?;
        let name = self.as_written(&name);
        self.agree_article(&name);
        self.written.push_str(&name);
        Ok(())
    }

    /// The clause owns the space before it, so a clause that goes takes that space along.
    fn add_clause(&mut self, clause: &Clause) -> Result<(), StepError> {
        if self.context.left_out.contains(&clause.part) {
            self.written.truncate(self.written.trim_end().len());
            return Ok(());
        }
        self.add_pieces(&clause.pieces)
    }

    fn as_written(&self, name: &str) -> String {
        if at_sentence_start(&self.written) {
            return with_first_letter(name, char::to_uppercase);
        }
        let first_word = name.split(' ').next().unwrap_or_default();
        if self.context.proper_name_words.iter().any(|word| word == first_word) {
            return name.to_string();
        }
        with_first_letter(name, char::to_lowercase)
    }

    /// An `a` or `an` just before a placeholder is chosen again from the first letter of the name in use.
    fn agree_article(&mut self, name: &str) {
        let Some(found) = article_before(&self.written) else {
            return;
        };
        let wanted = if starts_with_vowel(name) { "an" } else { "a" };
        let article = if found.capital { with_first_letter(wanted, char::to_uppercase) } else { wanted.to_string() };
        self.written.truncate(found.start);
        self.written.push_str(&article);
        self.written.push_str(&found.space);
    }
}

struct ArticleBefore {
    start: usize,
    capital: bool,
    space: String,
}

fn article_before(written: &str) -> Option<ArticleBefore> {
    let body = written.trim_end();
    let space = &written[body.len()..];
    if space.is_empty() {
        return None;
    }
    let start = [2, 1].into_iter().filter_map(|length| article_start(body, length)).next()?;
    let capital = body[start..].starts_with('A');
    Some(ArticleBefore { start, capital, space: space.to_string() })
}

/// Where an article of this many letters begins at the end of `body`, when it stands alone as a word.
fn article_start(body: &str, length: usize) -> Option<usize> {
    let start = body.len().checked_sub(length)?;
    let article = body.get(start..)?;
    let expected = if length == 2 { ["an", "An"] } else { ["a", "A"] };
    if !expected.contains(&article) {
        return None;
    }
    let before = body[..start].chars().next_back();
    if before.is_some_and(|letter| letter.is_ascii_alphabetic()) {
        return None;
    }
    Some(start)
}

fn at_sentence_start(written: &str) -> bool {
    if written.is_empty() {
        return true;
    }
    let body = written.trim_end();
    if body.len() == written.len() {
        return false;
    }
    body.ends_with(['.', '!', '?'])
}

fn starts_with_vowel(name: &str) -> bool {
    name.chars().next().is_some_and(|letter| VOWELS.contains(letter))
}

fn with_first_letter<I: Iterator<Item = char>>(text: &str, change: fn(char) -> I) -> String {
    let mut letters = text.chars();
    let Some(first) = letters.next() else {
        return String::new();
    };
    change(first).chain(letters).collect()
}
