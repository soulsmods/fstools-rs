use std::{path::Path, sync::Arc};

use bevy::asset::io::{AssetReader, AssetReaderError, PathStream, Reader, VecReader};

use crate::asset_source::vfs::state::VfsState;

pub struct VfsAssetReader {
    state: Arc<VfsState>,
}

impl VfsAssetReader {
    pub fn new(state: Arc<VfsState>) -> Self {
        Self { state }
    }
}

impl AssetReader for VfsAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;

        let bytes = self
            .state
            .read(path_str)
            .await
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;

        Ok(VecReader::new(bytes))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        Err::<VecReader, AssetReaderError>(AssetReaderError::NotFound(path.to_path_buf()))
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
