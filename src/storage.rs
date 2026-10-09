//! Where kept fridge photos go: a private Supabase Storage bucket.
//!
//! Only this service holds the key that can write or read the bucket. The
//! app never sees a stored photo's path or a link to it.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub type StoreFuture<'a> = Pin<Box<dyn Future<Output = Result<(), StoreError>> + Send + 'a>>;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("no photo storage is configured")]
    NotConfigured,
    #[error("the photo storage failed")]
    Failed,
}

pub trait PhotoStore: Send + Sync {
    fn is_configured(&self) -> bool;
    fn put<'a>(&'a self, file: StoredFile<'a>) -> StoreFuture<'a>;
    fn delete<'a>(&'a self, paths: &'a [String]) -> StoreFuture<'a>;
}

/// A file to store at a path in the bucket.
pub struct StoredFile<'a> {
    pub path: &'a str,
    pub bytes: Vec<u8>,
    pub mime_type: &'a str,
}

/// No bucket configured: the dataset consent cannot be given.
pub struct NoStore;

impl PhotoStore for NoStore {
    fn is_configured(&self) -> bool {
        false
    }

    fn put<'a>(&'a self, _file: StoredFile<'a>) -> StoreFuture<'a> {
        Box::pin(async { Err(StoreError::NotConfigured) })
    }

    fn delete<'a>(&'a self, _paths: &'a [String]) -> StoreFuture<'a> {
        Box::pin(async { Err(StoreError::NotConfigured) })
    }
}

pub struct SupabaseStore {
    http: reqwest::Client,
    /// `https://<project>.supabase.co/storage/v1/object/<bucket>`
    bucket_url: String,
    secret_key: String,
}

impl SupabaseStore {
    pub fn new(project_url: &str, bucket: &str, secret_key: String) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(30)).build()?;
        let base = project_url.trim_end_matches('/');
        let bucket_url = format!("{base}/storage/v1/object/{bucket}");
        Ok(SupabaseStore { http, bucket_url, secret_key })
    }

    fn authorized(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request.header("apikey", &self.secret_key).bearer_auth(&self.secret_key)
    }
}

impl PhotoStore for SupabaseStore {
    fn is_configured(&self) -> bool {
        true
    }

    fn put<'a>(&'a self, file: StoredFile<'a>) -> StoreFuture<'a> {
        let request = self.http.post(format!("{}/{}", self.bucket_url, file.path));
        let request = self.authorized(request).header("content-type", file.mime_type).body(file.bytes);
        Box::pin(send(request))
    }

    fn delete<'a>(&'a self, paths: &'a [String]) -> StoreFuture<'a> {
        let request = self.http.delete(&self.bucket_url);
        let request = self.authorized(request).json(&serde_json::json!({ "prefixes": paths }));
        Box::pin(send(request))
    }
}

async fn send(request: reqwest::RequestBuilder) -> Result<(), StoreError> {
    let response = request.send().await.map_err(|_| StoreError::Failed)?;
    if !response.status().is_success() {
        log_failure(response.status().as_u16());
        return Err(StoreError::Failed);
    }
    Ok(())
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_failure(status: u16) {
    tracing::warn!(status, "photo storage refused a request");
}
