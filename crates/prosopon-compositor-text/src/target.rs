//! Output targets for the text compositor.
//!
//! A `TextTarget` is any writeable sink — stdout, a file, or an in-memory buffer for
//! tests. Internally it's an `Arc<Mutex<dyn Write>>` so the compositor can render to
//! the same sink from multiple entry points (e.g. event apply + streaming chunk).

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// A cloneable writeable sink.
#[derive(Clone)]
pub struct TextTarget {
    inner: Arc<Mutex<Box<dyn Write + Send>>>,
    captured: Option<Arc<Mutex<Vec<u8>>>>,
}

impl TextTarget {
    /// Write to standard output.
    #[must_use]
    pub fn stdout() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Box::new(io::stdout()))),
            captured: None,
        }
    }

    /// Write to standard error.
    #[must_use]
    pub fn stderr() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Box::new(io::stderr()))),
            captured: None,
        }
    }

    /// Write into an in-memory buffer accessible via [`TextTarget::captured`].
    #[must_use]
    pub fn in_memory() -> Self {
        let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let cloned = Arc::clone(&buf);
        Self {
            inner: Arc::new(Mutex::new(Box::new(BufWriter(cloned)))),
            captured: Some(buf),
        }
    }

    /// Wrap any `Write + Send` target.
    pub fn from_writer<W: Write + Send + 'static>(w: W) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Box::new(w))),
            captured: None,
        }
    }

    /// Snapshot of bytes written to an in-memory target, if any.
    #[must_use]
    pub fn captured(&self) -> String {
        self.captured
            .as_ref()
            .map(|b| String::from_utf8_lossy(&b.lock().unwrap()).to_string())
            .unwrap_or_default()
    }
}

impl Write for TextTarget {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner
            .lock()
            .map_err(|_| io::Error::other("target mutex poisoned"))?
            .write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner
            .lock()
            .map_err(|_| io::Error::other("target mutex poisoned"))?
            .flush()
    }
}

struct BufWriter(Arc<Mutex<Vec<u8>>>);

impl Write for BufWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("in-memory mutex poisoned"))?
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
