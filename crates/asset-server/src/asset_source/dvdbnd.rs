use std::{io, path::Path, sync::Arc};

use bevy::asset::{
    io::{AssetReader, AssetReaderError, PathStream, Reader},
    BoxedFuture,
};
use fstools_dvdbnd::{DvdBnd, DvdBndEntryError};
use fstools_formats::dcx::DcxHeader;

use crate::asset_source::SimpleReader;

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
                .map_err(|err| match err {
                    DvdBndEntryError::NotFound => AssetReaderError::NotFound(path.to_path_buf()),
                    err => AssetReaderError::Io(Arc::new(io::Error::other(err))),
                })
                .and_then(|r| {
                    let is_dcx = {
                        let bytes = r.data();
                        &bytes[..4] == b"DCX\0"
                    };

                    let reader = if is_dcx {
                        let (_dcx_header, dcx_reader) = DcxHeader::read(r)
                            .map_err(|err| AssetReaderError::Io(Arc::new(io::Error::other(err))))?;

                        Box::new(SimpleReader(dcx_reader)) as Box<Reader>
                    } else {
                        Box::new(SimpleReader(r)) as Box<Reader>
                    };

                    Ok(reader)
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
