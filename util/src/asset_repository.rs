use std::collections;
use format::{bnd4::{File, FromBnd4File, BND4}, dcx::DCX};
use crate::{AssetArchive, AssetArchiveError};

#[derive(Default, Debug)]
pub struct AssetRepository {
    archives: collections::HashMap<String, AssetArchive>,
    binders: collections::HashMap<String, BND4>,
    file_handles: collections::HashMap<String, FileHandle>,
}

impl AssetRepository {
    pub fn mount_archive(
        &mut self,
        path: &str,
        key: &[u8],
    ) -> Result<(), AssetArchiveError> {
        self.archives.insert(
            path.to_string(),
            AssetArchive::new(path, key)?
        );

        Ok(())
    }

    pub fn mount_dcx_bnd4(&mut self, path: &str) -> Result<(), AssetArchiveError> {
        let file_bytes = self.file_bytes_by_path(path)?;

        // Build DCX seek table
        let mut dcx_cursor = std::io::Cursor::new(file_bytes);
        let dcx = DCX::from_reader(&mut dcx_cursor)
            .map_err(AssetArchiveError::DCX)?;

        let mut bnd4_cursor = std::io::Cursor::new(dcx.decompressed);
        let bnd4 = BND4::from_reader(&mut bnd4_cursor)
            .map_err(AssetArchiveError::BND4)?;

        for descriptor in bnd4.files.iter() {
            self.file_handles.insert(
                descriptor.path.to_string(),
                FileHandle::from(descriptor, path),
            );
        }

        self.binders.insert(path.to_string(), bnd4);

        Ok(())
    }

    pub fn paths_by_extension(&self, extension: &str) -> Vec<&FileHandle> {
        self.file_handles.iter()
            .filter(|h| h.0.ends_with(extension))
            .map(|h| h.1)
            .collect()
    }

    pub fn file_bytes(&self, handle: &FileHandle) -> &[u8] {
        let binder = &self.binders[&handle.container];
        let start = handle.offset as usize;
        let end = start + handle.size as usize;

        &binder.data[start..end]
    }

    pub fn file<TFile: FromBnd4File>(
        &self,
        handle: &FileHandle,
    ) -> TFile {
        TFile::from_bnd4(self.file_bytes(handle))
    }

    fn file_bytes_by_path(&self, path: &str) -> Result<Vec<u8>, AssetArchiveError> {
        for archive in self.archives.values() {
            match archive.file_bytes_by_path(path) {
                Ok(b) => return Ok(b),
                Err(e) => match e {
                    AssetArchiveError::FileNotFound => {},
                    _ => return Err(e),
                },
            }
        }
        Err(AssetArchiveError::FileNotFound)
    }
}

#[derive(Debug)]
pub struct FileHandle {
    /// The path of the wrapping DCX in the BHD
    container: String,

    /// Offset relative to the bnd4 data buffer
    offset: u64,

    /// Size of the uncompressed data
    size: u64,
}

impl FileHandle {
    pub fn from(
        file: &File,
        container: &str,
    ) -> Self {
        Self {
            container: container.to_string(),
            offset: file.data_offset as u64,
            size: file.compressed_size,
        }
    }
}
