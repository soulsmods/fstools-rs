use std::{io, io::Read, path::Path, pin::Pin, sync::Arc, task::Poll};

use bevy_asset::{
    io::{AssetReader, AssetReaderError, PathStream, Reader},
    BoxedFuture,
};
use fstools_dvdbnd::{DvdBnd, DvdBndEntryError, DvdBndEntryReader};
use futures_lite::AsyncRead;

#[derive(Clone)]
pub struct DvdBndAssetSource(pub(crate) Arc<DvdBnd>);

impl AssetReader for DvdBndAssetSource {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let path_str = path.to_string_lossy();
            let dvd_bnd = &self.0;

            dvd_bnd
                .open(&*path_str)
                .map(|r| Box::new(AsyncDvdBndEntryReader(r)) as Box<Reader>)
                .map_err(|e| match e {
                    DvdBndEntryError::NotFound => AssetReaderError::NotFound(path.to_path_buf()),
                    _ => AssetReaderError::Io(Arc::new(io::Error::other(
                        "failed to get data from DVDBND",
                    ))),
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

struct AsyncDvdBndEntryReader(DvdBndEntryReader);

impl AsyncRead for AsyncDvdBndEntryReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(self.0.read(buf))
    }
}
