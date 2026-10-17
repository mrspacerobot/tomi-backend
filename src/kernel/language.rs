//! The language the cook reads, from the app's `Accept-Language` header.

#[cfg(test)]
mod tests;

const ENGLISH: &str = "en";

/// A primary language code such as `en` or `de`. English when the header
/// is missing or cannot be read; content without the language falls back
/// to English too.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language(String);

impl Language {
    pub fn english() -> Self {
        Language(ENGLISH.to_string())
    }

    /// The first language the app asks for, such as `de` from `de-CH,de;q=0.9`.
    pub fn from_accept_language(header: Option<&str>) -> Self {
        let Some(header) = header else {
            return Language::english();
        };
        match primary_code(header) {
            Some(code) => Language(code),
            None => Language::english(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn primary_code(header: &str) -> Option<String> {
    let first = header.split(',').next()?;
    let tag = first.split(';').next()?.trim();
    let code = tag.split('-').next()?.to_ascii_lowercase();
    let looks_like_a_code = (2..=3).contains(&code.len()) && code.chars().all(|letter| letter.is_ascii_lowercase());
    if looks_like_a_code {
        return Some(code);
    }
    None
}
