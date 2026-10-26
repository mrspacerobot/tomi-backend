//! Captures what the server logs during one call, so a test can check what reaches the logs.

use std::io::{self, Write};
use std::sync::{Arc, Mutex, Once};

#[derive(Clone, Default)]
pub struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl LogBuffer {
    pub fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + 'static {
        // A permanent registry keeps callsite interest enabled while parallel scoped captures come and go.
        static LOGGING: Once = Once::new();
        LOGGING.call_once(|| tracing::subscriber::set_global_default(tracing_subscriber::registry()).unwrap());
        let writer = self.clone();
        tracing_subscriber::fmt().json().with_writer(move || writer.clone()).finish()
    }

    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
