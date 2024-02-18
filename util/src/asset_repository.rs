use std::collections;
use std::error::Error;
use std::io::Read;

use format::{
    bnd4::{BND4, File, FromBnd4File},
    dcx::DCX,
};
use souls_vfs::Vfs;

pub struct AssetRepository {
    vfs: Vfs,
    binders: collections::HashMap<String, BND4>,
    file_handles: collections::HashMap<String, FileHandle>,
}

impl AssetRepository {
    pub fn new(vfs: Vfs) -> Self {
        Self {
            vfs,
            binders: Default::default(),
            file_handles: Default::default(),
        }
    }

    pub fn mount_dcx_bnd4(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file_bytes = self.file_bytes_by_path(path)?;

        // Build DCX seek table
        let mut dcx_cursor = std::io::Cursor::new(file_bytes);
        let dcx = DCX::from_reader(&mut dcx_cursor)?;

        let mut bnd4_cursor = std::io::Cursor::new(dcx.decompressed);
        let bnd4 = BND4::from_reader(&mut bnd4_cursor)?;

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
        self.file_handles
            .iter()
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

    pub fn file<TFile: FromBnd4File>(&self, handle: &FileHandle) -> TFile {
        TFile::from_bnd4(self.file_bytes(handle))
    }

    fn file_bytes_by_path(&self, path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut file_reader = self.vfs.open(path)?;
        let mut file_data = Vec::new();

        file_reader.read_to_end(&mut file_data)?;

        Ok(file_data)
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
    pub fn from(file: &File, container: &str) -> Self {
        Self {
            container: container.to_string(),
            offset: file.data_offset as u64,
            size: file.compressed_size,
        }
    }
}
