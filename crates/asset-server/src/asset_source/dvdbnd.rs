use std::{
    io::{self, Read, SeekFrom},
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};

use bevy::{
    asset::io::{AssetReader, AssetReaderError, PathStream, Reader, VecReader},
    prelude::Deref,
};
use fstools_dvdbnd::{DvdBnd, DvdBndEntryError};
use fstools_formats::dcx::DcxHeader;
use futures_io::AsyncSeek;
use futures_lite::AsyncRead;
use memmap2::Mmap;

#[derive(Clone, Deref)]
pub struct DvdBndAssetSource(pub(crate) Arc<DvdBnd>);

/// An [`AsyncRead`] implementation capable of reading a [`Mmap`].
pub struct MmapReader {
    mmap: Mmap,
    bytes_read: u64,
}

impl Reader for MmapReader {
    fn seekable(
        &mut self,
    ) -> Result<&mut dyn bevy::asset::io::SeekableReader, bevy::asset::io::ReaderNotSeekableError>
    {
        Ok(self)
    }
}

impl MmapReader {
    /// Create a new [`MmapReader`] for `mmap`.
    pub fn new(mmap: Mmap) -> Self {
        Self {
            mmap,
            bytes_read: 0,
        }
    }
}

impl AsyncRead for MmapReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<futures_io::Result<usize>> {
        if self.bytes_read >= self.mmap.len() as u64 {
            Poll::Ready(Ok(0))
        } else {
            let n =
                ready!(Pin::new(&mut &self.mmap[self.bytes_read as usize..]).poll_read(cx, buf))?;
            self.bytes_read += n as u64;
            Poll::Ready(Ok(n))
        }
    }
}
impl AsyncSeek for MmapReader {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        seek: SeekFrom,
    ) -> Poll<std::io::Result<u64>> {
        let result = match seek {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::Current(offset) => self.bytes_read.checked_add_signed(offset),
            SeekFrom::End(offset) => (self.mmap.len() as u64).checked_sub_signed(offset),
        };

        if let Some(new_pos) = result {
            self.bytes_read = new_pos;
            Poll::Ready(Ok(new_pos))
        } else {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek position is out of range",
            )))
        }
    }
}

impl AssetReader for DvdBndAssetSource {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let path_str = path.to_string_lossy();
        let file = self.open(&*path_str).map_err(|err| match err {
            DvdBndEntryError::NotFound => AssetReaderError::NotFound(path.to_path_buf()),
            err => AssetReaderError::Io(Arc::new(io::Error::other(err))),
        })?;

        let is_dcx = { file.data().starts_with(b"DCX\0") };
        let reader: Box<dyn Reader> = if is_dcx {
            let (header, mut reader) = DcxHeader::read(file).map_err(std::io::Error::other)?;
            let data = blocking::unblock(move || {
                let mut data = Vec::with_capacity(header.sizes().decompressed() as usize);
                reader.read_to_end(&mut data)?;

                Ok::<_, std::io::Error>(data)
            })
            .await?;

            Box::from(VecReader::new(data))
        } else {
            Box::from(MmapReader::new(file.into()))
        };

        Ok(reader)
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<MmapReader, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }
}
