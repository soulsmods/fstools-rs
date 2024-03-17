use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use bevy::{
    asset::{
        io::{AssetReader, AssetReaderError, PathStream, Reader},
        BoxedFuture,
    },
    log::info,
    prelude::{Deref, DerefMut, Resource},
};
use crossbeam_channel::Sender;
use memmap2::{Mmap, MmapOptions};
use typed_path::Utf8WindowsPathBuf;

use crate::SimpleReader;

pub mod watcher;

#[derive(Clone, Resource)]
pub struct Vfs {
    inner: Arc<RwLock<VfsInner>>,
    event_sender: Sender<VfsEvent>,
}

pub enum VfsEvent {
    Added(PathBuf),
}

#[derive(Default)]
pub struct VfsInner {
    entries: HashMap<String, Mmap>,
}

impl Vfs {
    pub fn new(event_sender: Sender<VfsEvent>) -> Self {
        Self {
            event_sender,
            inner: Default::default(),
        }
    }

    pub fn mount_file(&mut self, name: String, data: Vec<u8>) {
        let mut inner = self.inner.write().expect("vfs_write_lock");

        // TODO: this is specific to Elden Ring
        let path = Utf8WindowsPathBuf::from(&name);
        let normalized_path = path
            .strip_prefix("N:/GR/data/INTERROOT_win64")
            .map(|path| path.with_unix_encoding().into_string())
            .expect("path_not_expected");

        info!("Mounting {normalized_path} into vfs");

        let _ = self
            .event_sender
            .send(VfsEvent::Added(PathBuf::from(&normalized_path)));

        let mut mmap = MmapOptions::default()
            .len(data.len())
            .map_anon()
            .expect("failed to allocate memory");
        mmap.copy_from_slice(&data[..]);

        inner.entries.insert(
            normalized_path,
            mmap.make_read_only()
                .expect("failed to make memory read-only"),
        );
    }

    pub fn entry_bytes<P: AsRef<str>>(&self, name: P) -> Option<&[u8]> {
        let inner = self.inner.read().expect("vfs_read_lock");

        inner.entries.get(name.as_ref()).map(|item| {
            let ptr = item.as_ptr();
            let len = item.len();

            // SAFETY: Pointer cannot be moved and is placed on the heap for the lifetime of
            // `self`.
            unsafe { std::slice::from_raw_parts(ptr, len) }
        })
    }
}

#[derive(Deref, DerefMut)]
pub struct VfsAssetSource(pub(crate) Vfs);

impl AssetReader for VfsAssetSource {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let bytes = self.entry_bytes(path.to_str().expect("invalid path"));

            match bytes {
                Some(data) => Ok(Box::new(SimpleReader(data)) as Box<Reader>),
                None => Err(AssetReaderError::NotFound(path.to_path_buf())),
            }
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(async move { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(async move { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }
}
