//! Reads a step text into text, placeholders and clauses.

use super::StepError;

/// `{part}`, `{part.ingredient}`, either with `:plural`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngredientPlaceholder {
    pub part: String,
    pub ingredient: Option<String>,
    pub plural: bool,
}

/// Text that holds only while its part is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub part: String,
    pub pieces: Vec<StepFragment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepFragment {
    Text(String),
    IngredientPlaceholder(IngredientPlaceholder),
    Clause(Clause),
}

enum Token {
    IngredientPlaceholder(IngredientPlaceholder),
    Open(String),
    Close,
}

/// The pieces of a text, or why it is not written in the step format.
pub fn parse(text: &str) -> Result<Vec<StepFragment>, StepError> {
    let mut parser = Parser { pieces: Vec::new(), open: None };
    let mut rest = text;
    while let Some(at) = rest.find(['{', '}', '[', ']']) {
        parser.add(StepFragment::Text(rest[..at].to_string()));
        let (token, length) = token_at(&rest[at..])?;
        parser.take(token)?;
        rest = &rest[at + length..];
    }
    parser.add(StepFragment::Text(rest.to_string()));
    parser.finish()
}

struct Parser {
    pieces: Vec<StepFragment>,
    open: Option<Clause>,
}

impl Parser {
    fn add(&mut self, piece: StepFragment) {
        if matches!(&piece, StepFragment::Text(text) if text.is_empty()) {
            return;
        }
        match &mut self.open {
            Some(clause) => clause.pieces.push(piece),
            None => self.pieces.push(piece),
        }
    }

    fn take(&mut self, token: Token) -> Result<(), StepError> {
        match token {
            Token::IngredientPlaceholder(mention) => self.add(StepFragment::IngredientPlaceholder(mention)),
            Token::Open(part) => self.open_clause(part)?,
            Token::Close => self.close_clause()?,
        }
        Ok(())
    }

    fn open_clause(&mut self, part: String) -> Result<(), StepError> {
        if self.open.is_some() {
            return Err(StepError::Malformed("a clause starts inside a clause"));
        }
        self.open = Some(Clause { part, pieces: Vec::new() });
        Ok(())
    }

    fn close_clause(&mut self) -> Result<(), StepError> {
        let Some(clause) = self.open.take() else {
            return Err(StepError::Malformed("a `]` closes no clause"));
        };
        self.pieces.push(StepFragment::Clause(clause));
        Ok(())
    }

    fn finish(self) -> Result<Vec<StepFragment>, StepError> {
        if self.open.is_some() {
            return Err(StepError::Malformed("a clause is never closed"));
        }
        Ok(self.pieces)
    }
}

/// The token that starts `rest` (which starts with a bracket) and how many bytes it takes.
fn token_at(rest: &str) -> Result<(Token, usize), StepError> {
    match rest.as_bytes()[0] {
        b'{' => mention_token(rest),
        b'[' => clause_token(rest),
        b']' => Ok((Token::Close, 1)),
        _ => Err(StepError::Malformed("a stray bracket")),
    }
}

fn mention_token(rest: &str) -> Result<(Token, usize), StepError> {
    let stray = StepError::Malformed("a stray `{` or a body that is no placeholder");
    let Some(end) = rest.find('}') else {
        return Err(stray);
    };
    let body = &rest[1..end];
    if body.contains(['{', '[', ']']) {
        return Err(stray);
    }
    match mention_of(body) {
        Some(mention) => Ok((Token::IngredientPlaceholder(mention), end + 1)),
        None => Err(stray),
    }
}

fn mention_of(body: &str) -> Option<IngredientPlaceholder> {
    let (named, plural) = match body.strip_suffix(":plural") {
        Some(named) => (named, true),
        None => (body, false),
    };
    let (part, ingredient) = match named.split_once('.') {
        Some((part, ingredient)) => (part, Some(ingredient)),
        None => (named, None),
    };
    if !is_slug(part) || !ingredient.is_none_or(is_slug) {
        return None;
    }
    Some(IngredientPlaceholder { part: part.to_string(), ingredient: ingredient.map(str::to_string), plural })
}

fn clause_token(rest: &str) -> Result<(Token, usize), StepError> {
    let name_length = rest[1..].find(|letter: char| !is_slug_letter(letter)).unwrap_or(rest.len() - 1);
    let opening = &rest[1 + name_length..];
    if name_length == 0 || !opening.starts_with(": ") {
        return Err(StepError::Malformed("a stray `[` or a clause with no part"));
    }
    Ok((Token::Open(rest[1..1 + name_length].to_string()), 1 + name_length + 2))
}

fn is_slug(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_slug_letter)
}

fn is_slug_letter(letter: char) -> bool {
    letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '-'
}
