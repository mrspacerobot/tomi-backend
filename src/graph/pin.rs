//! `graph.lock`: the one graph version and checksum this build may serve.

const NONE: &str = "none";
const SHA256_LETTERS: usize = 64;

/// The graph this build is pinned to. Changing the pin is a reviewed change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphPin {
    pub version: String,
    pub sha256: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("graph.lock: {0}")]
pub struct PinError(String);

impl GraphPin {
    /// `none` pins no graph. Anything else must name a version and a checksum,
    /// each once. A lock that cannot be read is an error, never "no graph".
    pub fn parse(text: &str) -> Result<Option<GraphPin>, PinError> {
        let lines = meaningful_lines(text);
        if lines == [NONE] {
            return Ok(None);
        }
        if lines.len() != 2 {
            return Err(PinError("expected `none`, or a version line and a sha256 line".into()));
        }
        let sha256 = value_of(&lines, "sha256")?;
        check_sha256(&sha256)?;
        Ok(Some(GraphPin { version: value_of(&lines, "version")?, sha256 }))
    }
}

fn meaningful_lines(text: &str) -> Vec<&str> {
    let trimmed = text.lines().map(str::trim);
    trimmed.filter(|line| !line.is_empty() && !line.starts_with('#')).collect()
}

fn value_of(lines: &[&str], key: &str) -> Result<String, PinError> {
    let prefix = format!("{key} ");
    let mut found = lines.iter().filter_map(|line| line.strip_prefix(prefix.as_str()));
    match (found.next(), found.next()) {
        (Some(value), None) if !value.trim().is_empty() => Ok(value.trim().to_string()),
        _ => Err(PinError(format!("expected one `{key}` line with a value"))),
    }
}

fn check_sha256(text: &str) -> Result<(), PinError> {
    let lowercase_hex = text.chars().all(|letter| matches!(letter, '0'..='9' | 'a'..='f'));
    if text.len() == SHA256_LETTERS && lowercase_hex {
        return Ok(());
    }
    Err(PinError("the sha256 must be 64 lowercase hex letters".into()))
}
