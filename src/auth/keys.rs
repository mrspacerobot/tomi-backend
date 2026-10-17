//! Supabase's public signing keys, fetched and kept by key id.

use super::AuthError;
use jsonwebtoken::jwk::{AlgorithmParameters, Jwk, JwkSet, KeyAlgorithm};
use jsonwebtoken::{Algorithm, DecodingKey};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A token with an unknown key id triggers a refetch, but not more often
/// than this, counted from the last attempt whether or not it worked, so
/// forged tokens cannot make us hammer Supabase or queue everyone behind it.
const REFETCH_GAP: Duration = Duration::from_secs(60);
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(test)]
mod tests;

type KeysById = HashMap<String, VerifyingKey>;

/// A public key and the one algorithm it may be used with.
#[derive(Clone)]
pub struct VerifyingKey {
    pub key: DecodingKey,
    pub algorithm: Algorithm,
}

pub struct KeySet {
    source: Option<Source>,
    cache: RwLock<Cache>,
}

struct Source {
    url: String,
    http: reqwest::Client,
}

#[derive(Default)]
struct Cache {
    keys: KeysById,
    last_attempt: Option<Instant>,
    last_attempt_failed: bool,
}

impl KeySet {
    pub fn fetched_from(url: String) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder().timeout(FETCH_TIMEOUT).build()?;
        let source = Source { url, http };
        Ok(KeySet { source: Some(source), cache: RwLock::new(Cache::default()) })
    }

    pub fn fixed(jwks: &JwkSet) -> Self {
        let cache = Cache { keys: usable_keys(jwks), ..Cache::default() };
        KeySet { source: None, cache: RwLock::new(cache) }
    }

    pub async fn find(&self, kid: &str) -> Result<VerifyingKey, AuthError> {
        if let Some(found) = self.cache.read().await.keys.get(kid) {
            return Ok(found.clone());
        }
        let Some(source) = &self.source else {
            return Err(AuthError::Invalid);
        };
        let mut cache = self.cache.write().await;
        refresh_if_stale(&mut cache, source).await?;
        let Some(found) = cache.keys.get(kid) else {
            return Err(AuthError::Invalid);
        };
        Ok(found.clone())
    }
}

async fn refresh_if_stale(cache: &mut Cache, source: &Source) -> Result<(), AuthError> {
    if let Some(last_attempt) = cache.last_attempt {
        if last_attempt.elapsed() < REFETCH_GAP {
            return recent_outcome(cache);
        }
    }
    cache.last_attempt = Some(Instant::now());
    let fetched = fetch(source).await;
    cache.last_attempt_failed = fetched.is_err();
    cache.keys = usable_keys(&fetched?);
    Ok(())
}

/// Inside the gap nothing is fetched. A recent failure is still a failure: the
/// keys are unavailable, which is not the same as the token being invalid.
fn recent_outcome(cache: &Cache) -> Result<(), AuthError> {
    if cache.last_attempt_failed {
        return Err(AuthError::KeysUnavailable);
    }
    Ok(())
}

async fn fetch(source: &Source) -> Result<JwkSet, AuthError> {
    let unavailable = |error: reqwest::Error| {
        log_fetch_failure(&error);
        AuthError::KeysUnavailable
    };
    let response = source.http.get(&source.url).send().await.map_err(unavailable)?;
    let response = response.error_for_status().map_err(unavailable)?;
    response.json().await.map_err(unavailable)
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_fetch_failure(error: &reqwest::Error) {
    tracing::warn!(%error, "could not fetch signing keys");
}

/// The keys we can use. A key without an id, or with an algorithm we do
/// not accept, is left out.
fn usable_keys(jwks: &JwkSet) -> KeysById {
    let mut keys = HashMap::new();
    for jwk in &jwks.keys {
        if let Some((kid, key)) = usable_key(jwk) {
            keys.insert(kid, key);
        }
    }
    keys
}

fn usable_key(jwk: &Jwk) -> Option<(String, VerifyingKey)> {
    let kid = jwk.common.key_id.clone()?;
    let algorithm = algorithm_of(jwk)?;
    let key = DecodingKey::from_jwk(jwk).ok()?;
    Some((kid, VerifyingKey { key, algorithm }))
}

/// The algorithm a key declares, or else the usual one for its key type.
fn algorithm_of(jwk: &Jwk) -> Option<Algorithm> {
    match jwk.common.key_algorithm {
        Some(declared) => accepted(declared),
        None => usual_for(&jwk.algorithm),
    }
}

fn accepted(declared: KeyAlgorithm) -> Option<Algorithm> {
    match declared {
        KeyAlgorithm::ES256 => Some(Algorithm::ES256),
        KeyAlgorithm::RS256 => Some(Algorithm::RS256),
        _ => None,
    }
}

fn usual_for(parameters: &AlgorithmParameters) -> Option<Algorithm> {
    match parameters {
        AlgorithmParameters::EllipticCurve(_) => Some(Algorithm::ES256),
        AlgorithmParameters::RSA(_) => Some(Algorithm::RS256),
        _ => None,
    }
}
