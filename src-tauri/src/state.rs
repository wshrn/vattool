use std::sync::Arc;
use tokio::sync::Mutex;

pub struct FileWriteLock(pub Arc<Mutex<()>>);

impl Default for FileWriteLock {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

impl FileWriteLock {
    pub fn clone_arc(&self) -> Arc<Mutex<()>> {
        self.0.clone()
    }
}
