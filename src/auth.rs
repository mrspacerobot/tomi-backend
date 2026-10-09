//! Checks Supabase access tokens.
//!
//! The app signs in with Supabase Auth and sends the access token it gets
//! as `Authorization: Bearer <token>`. Supabase signs tokens with an
//! asymmetric key published as a key set (see `keys`); older projects use
//! one shared HS256 secret. Both work, the shared secret mainly for tests.

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Header, Validation};
use serde::Deserialize;
use uuid::Uuid;

mod keys;
#[cfg(test)]
mod tests;

use keys::{KeySet, VerifyingKey};

/// Supabase puts this audience, and this role, on signed-in users' tokens.
const SIGNED_IN: &str = "authenticated";

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum AuthError {
    #[error("no bearer token")]
    Missing,
    #[error("token is not valid")]
    Invalid,
    #[error("anonymous sessions cannot use this API")]
    Anonymous,
    #[error("signing keys are unavailable")]
    KeysUnavailable,
}

/// The parts of the token the service uses.
#[derive(Debug, Deserialize)]
struct Claims {
    sub: Uuid,
    role: String,
    #[serde(default)]
    is_anonymous: bool,
}

pub struct Verifier {
    issuer: String,
    shared_secret: Option<DecodingKey>,
    keys: KeySet,
}

impl Verifier {
    /// `project_url` is the Supabase project URL, such as
    /// `https://abc.supabase.co`. The key set is fetched when first needed.
    pub fn new(project_url: &str, shared_secret: Option<&str>) -> Result<Self, reqwest::Error> {
        let base = project_url.trim_end_matches('/');
        let jwks_url = format!("{base}/auth/v1/.well-known/jwks.json");
        Ok(Verifier {
            issuer: format!("{base}/auth/v1"),
            shared_secret: shared_secret.map(|secret| DecodingKey::from_secret(secret.as_bytes())),
            keys: KeySet::fetched_from(jwks_url)?,
        })
    }

    /// For tests: a verifier that knows its keys and never fetches.
    pub fn with_keys(project_url: &str, jwks: &JwkSet) -> Self {
        let base = project_url.trim_end_matches('/');
        Verifier { issuer: format!("{base}/auth/v1"), shared_secret: None, keys: KeySet::fixed(jwks) }
    }

    /// Returns the Supabase user id of a valid token.
    pub async fn verify(&self, token: &str) -> Result<Uuid, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::Invalid)?;
        let key = self.key_for(header).await?;
        let claims = self.decode_claims(token, &key)?;
        signed_in_user(claims)
    }

    async fn key_for(&self, header: Header) -> Result<VerifyingKey, AuthError> {
        if header.alg == Algorithm::HS256 {
            return self.shared_key();
        }
        let Some(kid) = header.kid else {
            return Err(AuthError::Invalid);
        };
        let found = self.keys.find(&kid).await?;
        if found.algorithm != header.alg {
            return Err(AuthError::Invalid);
        }
        Ok(found)
    }

    fn shared_key(&self) -> Result<VerifyingKey, AuthError> {
        let Some(secret) = &self.shared_secret else {
            return Err(AuthError::Invalid);
        };
        let key = secret.clone();
        Ok(VerifyingKey { key, algorithm: Algorithm::HS256 })
    }

    fn decode_claims(&self, token: &str, key: &VerifyingKey) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(key.algorithm);
        validation.set_audience(&[SIGNED_IN]);
        validation.set_issuer(&[&self.issuer]);
        validation.set_required_spec_claims(&["exp", "sub", "aud", "iss"]);
        let data = decode::<Claims>(token, &key.key, &validation).map_err(|_| AuthError::Invalid)?;
        Ok(data.claims)
    }
}

fn signed_in_user(claims: Claims) -> Result<Uuid, AuthError> {
    if claims.is_anonymous {
        return Err(AuthError::Anonymous);
    }
    if claims.role != SIGNED_IN {
        return Err(AuthError::Invalid);
    }
    Ok(claims.sub)
}

/// Takes the token out of an `Authorization: Bearer …` header value.
pub fn bearer(header: Option<&str>) -> Result<&str, AuthError> {
    let Some(value) = header else {
        return Err(AuthError::Missing);
    };
    let Some(token) = value.strip_prefix("Bearer ") else {
        return Err(AuthError::Missing);
    };
    let token = token.trim();
    if token.is_empty() {
        return Err(AuthError::Missing);
    }
    Ok(token)
}
