//! The checksum of an artifact: SHA-256, written as lowercase hex.

use sha2::{Digest, Sha256};

pub fn sha256_hex(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
