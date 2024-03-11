mod bnd4;

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use bevy_asset::{
    io::{AssetReader, AssetReaderError, PathStream, Reader}, BoxedFuture, UntypedHandle,
};
use futures_lite::io::Cursor;

pub struct ArchiveEntry {
    data: Vec<u8>,
}

pub struct ArchiveAssetSource {
    mounted_archives: HashSet<UntypedHandle>,
    entries: HashMap<String, ArchiveEntry>,
}

pub trait IntoArchive {
    fn files(&self) -> impl Iterator<Item = (String, Vec<u8>)>;
}

impl ArchiveAssetSource {
    pub fn mount<A: IntoArchive>(&mut self, archive: A) {
        self.entries.extend(archive.files().map(|(name, data)| {
            (
                name,
                ArchiveEntry {
                    data,
                },
            )
        }));
    }
}

impl AssetReader for ArchiveAssetSource {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let data = self
                .entries
                .get(&path.to_string_lossy().to_string())
                .map(|entry| Box::new(Cursor::new(&entry.data   )) as Box<Reader<'a>>)
                .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;

            Ok(data)
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
