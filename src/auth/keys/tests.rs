use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

const FAILURE: &str = "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";

/// A key server that always fails, and how many times it was asked.
async fn failing_key_server() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/jwks.json", listener.local_addr().unwrap());
    let asked = Arc::new(AtomicUsize::new(0));
    let counter = asked.clone();
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            counter.fetch_add(1, Ordering::SeqCst);
            let _ = socket.write_all(FAILURE.as_bytes()).await;
        }
    });
    (url, asked)
}

#[tokio::test]
async fn a_failed_fetch_is_not_repeated_for_every_unknown_key_id() {
    let (url, asked) = failing_key_server().await;
    let keys = KeySet::fetched_from(url).unwrap();
    for kid in ["a", "b", "c"] {
        assert!(matches!(keys.find(kid).await, Err(AuthError::KeysUnavailable)), "{kid}");
    }
    assert_eq!(asked.load(Ordering::SeqCst), 1);
}
