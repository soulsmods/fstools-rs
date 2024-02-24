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
            // Hack to trick bevy's extension matching
            let path_str = path.to_str().unwrap().replace("@", ".");
            let unfucked_path = std::path::Path::new(&path_str);
            let path = &unfucked_path;

            // Check the BHD first
            let from_bhd = self.open(path.to_string_lossy());
            if let Ok(bhd_reader) = from_bhd {
                let reader: Box<Reader> = Box::new(VfsEntryReader(bhd_reader));
                return Ok(reader);
            }

            // TODO: handle errors

            match self.open_from_mounts(path.to_str().unwrap()) {
                Ok(b) => {
                    let reader: Box<Reader> = Box::new(Cursor::new(b));
                    return Ok(reader);
                },
                Err(e) => Err(match e {
                    VfsOpenError::Mmap(e) => {
                        AssetReaderError::Io(e.into())
                    },
                    VfsOpenError::NotFound => {
                        AssetReaderError::NotFound(path.to_path_buf())
                    },
                }),
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
