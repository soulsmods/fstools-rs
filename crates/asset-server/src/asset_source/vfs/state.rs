use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::sync::{oneshot, RwLock};

pub(crate) enum EntryState {
    Ready(Vec<u8>),
    Waiting(oneshot::Sender<Vec<u8>>),
}

pub(crate) struct VfsInner {
    pub entries: HashMap<String, EntryState>,
}

impl VfsInner {
    pub fn insert_entry(&mut self, path: String, data: Vec<u8>) {
        if let Some(EntryState::Waiting(tx)) = self.entries.remove(&path) {
            let _ = tx.send(data);
        } else {
            self.entries.insert(path, EntryState::Ready(data));
        }
    }

    pub fn drain_waiters(&mut self) {
        self.entries
            .retain(|_, v| !matches!(v, EntryState::Waiting(_)));
    }
}

pub struct VfsState {
    pub(crate) inner: RwLock<VfsInner>,
    pub(crate) pending_mounts: AtomicU64,
}

impl VfsState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: RwLock::new(VfsInner {
                entries: HashMap::new(),
            }),
            pending_mounts: AtomicU64::new(0),
        })
    }

    pub async fn read(&self, path: &str) -> Option<Vec<u8>> {
        let rx = {
            let mut inner = self.inner.write().await;

            if let Some(EntryState::Ready(bytes)) = inner.entries.remove(path) {
                return Some(bytes);
            }

            if self.pending_mounts.load(Ordering::Acquire) == 0 {
                return None;
            }

            let (tx, rx) = oneshot::channel();
            inner
                .entries
                .insert(path.to_string(), EntryState::Waiting(tx));
            rx
        };

        rx.await.ok()
    }
}
