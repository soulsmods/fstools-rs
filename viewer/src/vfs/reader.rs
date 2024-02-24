use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader};
use bevy::asset::BoxedFuture;
use bevy::prelude::{Deref, DerefMut, Resource};
use bevy::tasks::futures_lite::io::Cursor;
use bevy::tasks::futures_lite::AsyncRead;
use souls_vfs::{Vfs, VfsEntryReader as VfsEntryReaderImpl, VfsOpenError};
use std::io::{self, Read};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

#[derive(Clone, Deref, DerefMut, Resource)]
pub struct VfsAssetRepository(pub(crate) Arc<Vfs>);

impl AssetReader for VfsAssetRepository {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let path_str = path.to_string_lossy();

            self.open(&path_str)
                .map(|r| Box::new(VfsEntryReader(r)) as Box<Reader>)
                .or_else(|_| Ok(
                    self.open_from_mounts(&path_str)
                        .map(|r| Box::new(Cursor::new(r)))?
                ))
                .map_err(|e| match e {
                    VfsOpenError::NotFound => AssetReaderError::NotFound(
                        path.to_path_buf()
                    ),
                })
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

struct VfsEntryReader<'a>(VfsEntryReaderImpl<'a>);

impl<'a> AsyncRead for VfsEntryReader<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.0.read(buf))
    }
}
