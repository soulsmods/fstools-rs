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
use typed_path::Utf8WindowsPathBuf;

use crate::SimpleReader;

mod bnd4_mount;
pub mod watcher;

pub trait IntoArchive {
    fn files(&self) -> impl Iterator<Item = (String, Vec<u8>)>;
}

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
    entries: HashMap<String, Box<[u8]>>,
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

        let path = Utf8WindowsPathBuf::from(&name);
        let filename = path
            .file_name()
            .expect("no filename")
            .to_string()
            .to_ascii_lowercase();

        info!("Mounting {filename} into vfs");

        let _ = self
            .event_sender
            .send(VfsEvent::Added(PathBuf::from(filename.clone())));

        inner.entries.insert(filename, data.into_boxed_slice());
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

impl VfsInner {}

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
