use std::{io, path::Path, sync::Arc};

use bevy::{
    asset::{
        io::{AssetReader, AssetReaderError, PathStream, Reader},
        BoxedFuture,
    },
    prelude::Deref,
};
use blocking::Unblock;
use fstools_dvdbnd::{DvdBnd, DvdBndEntryError};
use fstools_formats::dcx::DcxHeader;

use crate::asset_source::fast_path::FastPathReader;

#[derive(Clone, Deref)]
pub struct DvdBndAssetSource(pub(crate) Arc<DvdBnd>);

impl AssetReader for DvdBndAssetSource {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
            let path_str = path.to_string_lossy();
            let file = self.open(&*path_str).map_err(|err| match err {
                DvdBndEntryError::NotFound => AssetReaderError::NotFound(path.to_path_buf()),
                err => AssetReaderError::Io(Arc::new(io::Error::other(err))),
            })?;

            let is_dcx = { file.data().starts_with(b"DCX\0") };

            let reader = if is_dcx {
                let (_dcx_header, dcx_reader) = DcxHeader::read(file)
                    .map_err(|err| AssetReaderError::Io(Arc::new(io::Error::other(err))))?;

                FastPathReader::Reader(Box::new(Unblock::new(dcx_reader)))
            } else {
                FastPathReader::MemoryMapped(file.into(), 0)
            };

            Ok(Box::new(reader) as Box<Reader>)
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
