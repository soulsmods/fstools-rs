use std::{
    collections::HashMap,
    marker::PhantomPinned,
    path::{Path, PathBuf},
    pin::{pin, Pin},
    sync::{Arc, RwLock},
};

use bevy::{
    asset::{
        BoxedFuture,
        io::{AssetReader, AssetReaderError, PathStream, Reader},
    },
    log::{debug, info},
    prelude::{Deref, DerefMut, Resource},
};
use typed_path::{Utf8WindowsPathBuf, WindowsPath, WindowsPathBuf};

use crate::SimpleReader;

mod bnd4_mount;
pub trait IntoArchive {
    fn files(&self) -> impl Iterator<Item = (String, Vec<u8>)>;
}

#[derive(Clone, Default, Resource)]
pub struct Vfs {
    inner: Arc<RwLock<VfsInner>>,
}

#[derive(Default)]
pub struct VfsInner {
    entries: HashMap<String, Box<[u8]>>,
}

impl Vfs {
    pub fn mount_file(&mut self, name: String, data: Vec<u8>) {
        let mut inner = self.inner.write().expect("vfs_write_lock");

        let path = Utf8WindowsPathBuf::from(&name);
        let filename = path
            .file_name()
            .expect("no filename")
            .to_string()
            .to_ascii_lowercase();

        info!("Mounting {filename} into vfs");

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
        todo!()
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        todo!()
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        todo!()
    }
}
