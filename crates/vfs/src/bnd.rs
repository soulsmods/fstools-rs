use std::{
    collections::HashMap,
    io::{self, Cursor, Read},
};

use fstools_formats::{
    bnd4::BND4,
    dcx::{DcxError, DcxHeader},
};
use thiserror::Error;

use crate::{DvdBndEntryError, Name};

/// Provides easy access into a collection of BND4 archives.
#[derive(Default)]
pub struct BndMountHost {
    pub mounted: HashMap<Name, BndBytes>,
    pub entries: HashMap<String, BndFileEntry>,
}

#[derive(Debug, Error)]
pub enum BndMountError {
    #[error("Could not get vfs file reader: {0}")]
    VfsOpen(#[from] DvdBndEntryError),

    #[error("Could not get copy bnd4 bytes from vfs reader: {0}")]
    DataCopy(io::Error),

    #[error("Could not parse Dcx: {0}")]
    Dcx(#[from] DcxError),

    #[error("Could not parse BND4: {0}")]
    Bnd4(io::Error),
}

impl BndMountHost {
    pub fn mount(&mut self, name: Name, bytes: &[u8]) -> Result<(), BndMountError> {
        let decompressed = undo_container_compression(bytes)?;

        let mut cursor = Cursor::new(decompressed);
        let bnd = BND4::from_reader(&mut cursor).map_err(BndMountError::Bnd4)?;

        self.entries.extend(bnd.files.iter().map(|f| {
            (
                Self::extract_file_name(&f.path).to_ascii_lowercase(),
                BndFileEntry {
                    container: name.clone(),
                    offset: f.data_offset as usize,
                    size: f.compressed_size as usize,
                },
            )
        }));

        self.mounted.insert(name, BndBytes(bnd.data));

        Ok(())
    }

    fn entry_bytes(&self, entry: &BndFileEntry) -> Result<&[u8], DvdBndEntryError> {
        if let Some(mount) = self.mounted.get(&entry.container) {
            let start = entry.offset;
            let end = start + entry.size;

            Ok(&mount.0[start..end])
        } else {
            Err(DvdBndEntryError::NotFound)
        }
    }

    fn extract_file_name(path: &str) -> String {
        // TODO: figure out if this works for Windows systems
        let normalized = path.replace('\\', "/");
        let path = std::path::PathBuf::from(normalized);

        path.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn bytes_by_file_name(&self, name: &str) -> Result<&[u8], DvdBndEntryError> {
        let normalized_name = name.to_ascii_lowercase();
        let entry = self
            .entries
            .iter()
            .find(|(k, _)| **k == normalized_name)
            .ok_or(DvdBndEntryError::NotFound)?
            .1;

        self.entry_bytes(entry)
    }
}

pub struct BndBytes(Vec<u8>);

#[derive(Debug)]
pub struct BndFileEntry {
    container: Name,
    offset: usize,
    size: usize,
}

// Optionally undoes any Dcx compression when detected. Unfortunately there is
// no guarantee that any file will be Dcx compressed but they usually are
// meaning that the hot path will generally involve a copy.
// Optionally undoes any Dcx compression when detected. Unfortunately there is
// no guarantee that any file will be Dcx compressed but they usually are
// meaning that the hot path will generally involve a copy.
pub fn undo_container_compression(buf: &[u8]) -> Result<Vec<u8>, DcxError> {
    if DcxHeader::has_magic(buf) {
        let (_, mut decoder) = DcxHeader::read(buf)?;
        let mut decompressed = Vec::with_capacity(decoder.hint_size());
        decoder.read_to_end(&mut decompressed)?;

        Ok(decompressed)
    } else {
        Ok(buf.to_vec())
    }
}
