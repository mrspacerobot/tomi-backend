//! Checks Supabase access tokens.
//!
//! The app signs in with Supabase Auth and sends the access token it gets as
//! `Authorization: Bearer <token>`. Supabase signs tokens with an asymmetric
//! key published at `<project>/auth/v1/.well-known/jwks.json`; older projects
//! use one shared HS256 secret. Both are supported, the shared secret mainly
//! for local tests.

use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet, KeyAlgorithm};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Supabase puts this audience on tokens of signed-in users.
const AUDIENCE: &str = "authenticated";
/// A token with an unknown key id triggers a refetch, but not more often
/// than this, so forged tokens cannot make us hammer Supabase.
const REFETCH_GAP: Duration = Duration::from_secs(60);

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
    jwks_url: Option<String>,
    http: reqwest::Client,
    keys: RwLock<KeyCache>,
}

#[derive(Default)]
struct KeyCache {
    by_kid: HashMap<String, (DecodingKey, Algorithm)>,
    fetched_at: Option<Instant>,
}

impl Verifier {
    /// `supabase_url` is the project URL, such as `https://abc.supabase.co`.
    pub fn new(supabase_url: &str, shared_secret: Option<&str>, fetch_jwks: bool) -> Self {
        let base = supabase_url.trim_end_matches('/');
        Verifier {
            issuer: format!("{base}/auth/v1"),
            shared_secret: shared_secret.map(|secret| DecodingKey::from_secret(secret.as_bytes())),
            jwks_url: fetch_jwks.then(|| format!("{base}/auth/v1/.well-known/jwks.json")),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("http client"),
            keys: RwLock::new(KeyCache::default()),
        }
    }

    /// For tests: a verifier that knows its keys and never fetches.
    pub fn with_keys(supabase_url: &str, jwks: &JwkSet) -> Self {
        let verifier = Verifier::new(supabase_url, None, false);
        verifier.keys.try_write().expect("fresh lock").by_kid = keys_from(jwks);
        verifier
    }

    /// Returns the Supabase user id of a valid token.
    pub async fn verify(&self, token: &str) -> Result<Uuid, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::Invalid)?;
        let (key, algorithm) = match header.alg {
            Algorithm::HS256 => {
                let key = self.shared_secret.clone().ok_or(AuthError::Invalid)?;
                (key, Algorithm::HS256)
            }
            _ => {
                let kid = header.kid.ok_or(AuthError::Invalid)?;
                self.key_for(&kid).await?
            }
        };
        if algorithm != header.alg {
            return Err(AuthError::Invalid);
        }

        let mut validation = Validation::new(algorithm);
        validation.set_audience(&[AUDIENCE]);
        validation.set_issuer(&[&self.issuer]);
        validation.set_required_spec_claims(&["exp", "sub", "aud", "iss"]);
        let claims = decode::<Claims>(token, &key, &validation)
            .map_err(|_| AuthError::Invalid)?
            .claims;

        if claims.is_anonymous {
            return Err(AuthError::Anonymous);
        }
        if claims.role != AUDIENCE {
            return Err(AuthError::Invalid);
        }
        Ok(claims.sub)
    }

    async fn key_for(&self, kid: &str) -> Result<(DecodingKey, Algorithm), AuthError> {
        if let Some(found) = self.keys.read().await.by_kid.get(kid) {
            return Ok(found.clone());
        }
        let Some(url) = &self.jwks_url else {
            return Err(AuthError::Invalid);
        };

        let mut cache = self.keys.write().await;
        let recently = cache
            .fetched_at
            .is_some_and(|at| at.elapsed() < REFETCH_GAP);
        if !recently {
            let jwks = self.fetch(url).await?;
            cache.by_kid = keys_from(&jwks);
            cache.fetched_at = Some(Instant::now());
        }
        cache.by_kid.get(kid).cloned().ok_or(AuthError::Invalid)
    }

    async fn fetch(&self, url: &str) -> Result<JwkSet, AuthError> {
        let response = self.http.get(url).send().await.map_err(|error| {
            tracing::warn!(%error, "could not fetch signing keys");
            AuthError::KeysUnavailable
        })?;
        response
            .error_for_status()
            .map_err(|_| AuthError::KeysUnavailable)?
            .json()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "signing keys were unreadable");
                AuthError::KeysUnavailable
            })
    }
}

/// Keeps the keys we can use. A key without a kid or with an algorithm we
/// do not accept is skipped.
fn keys_from(jwks: &JwkSet) -> HashMap<String, (DecodingKey, Algorithm)> {
    let mut keys = HashMap::new();
    for jwk in &jwks.keys {
        let Some(kid) = jwk.common.key_id.clone() else {
            continue;
        };
        let Some(algorithm) = algorithm_of(jwk) else {
            continue;
        };
        let Ok(key) = DecodingKey::from_jwk(jwk) else {
            continue;
        };
        keys.insert(kid, (key, algorithm));
    }
    keys
}

fn algorithm_of(jwk: &jsonwebtoken::jwk::Jwk) -> Option<Algorithm> {
    match (&jwk.common.key_algorithm, &jwk.algorithm) {
        (Some(KeyAlgorithm::ES256), _) => Some(Algorithm::ES256),
        (Some(KeyAlgorithm::RS256), _) => Some(Algorithm::RS256),
        (Some(_), _) => None,
        (None, AlgorithmParameters::EllipticCurve(_)) => Some(Algorithm::ES256),
        (None, AlgorithmParameters::RSA(_)) => Some(Algorithm::RS256),
        (None, _) => None,
    }
}

/// Takes the token out of an `Authorization: Bearer …` header value.
pub fn bearer(header: Option<&str>) -> Result<&str, AuthError> {
    let value = header.ok_or(AuthError::Missing)?;
    let token = value
        .strip_prefix("Bearer ")
        .ok_or(AuthError::Missing)?
        .trim();
    if token.is_empty() {
        return Err(AuthError::Missing);
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_bearer_header() {
        assert_eq!(bearer(Some("Bearer abc")), Ok("abc"));
        assert_eq!(bearer(Some("Basic abc")), Err(AuthError::Missing));
        assert_eq!(bearer(Some("Bearer ")), Err(AuthError::Missing));
        assert_eq!(bearer(None), Err(AuthError::Missing));
    }
}
