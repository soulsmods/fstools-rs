use std::fs;
use std::io;
use std::io::{Read, Seek, SeekFrom};

use format::bhd::hash_path;
use format::bhd::FileDescriptor;
use format::bhd::{BHDError, BHD};

#[derive(Debug)]
pub enum AssetArchiveError {
    IO(io::Error),
    BHD(BHDError),
    FileNotFound,
    DCX(io::Error),
    BND4(io::Error),
}

#[derive(Debug)]
pub struct AssetArchive {
    header: BHD,
    data_path: String,
}

impl AssetArchive {
    pub fn new(path: &str, key: &[u8]) -> Result<Self, AssetArchiveError> {
        let mut header_file = fs::File::open(format!("{}.bhd", path))
            .map_err(AssetArchiveError::IO)?;

        let header = BHD::from_reader_with_key(
            &mut header_file,
            key,
        ).map_err(AssetArchiveError::BHD)?;

        Ok(Self {
            header,
            data_path: format!("{}.bdt", path),
        })
    }

    // TODO: maybe make this an iterator?
    pub fn files(&self) -> Vec<&FileDescriptor> {
        self.header.buckets.iter()
            .flat_map(|b| b.files.as_slice())
            .collect()
    }

    pub fn file_descriptor_by_path(&self, path: &str) -> Option<&FileDescriptor> {
        self.file_descriptor_by_hash(hash_path(path))
    }

    fn file_descriptor_by_hash(&self, hash: u64) -> Option<&FileDescriptor> {
        self.header.buckets.iter()
            .flat_map(|b| b.files.as_slice())
            .find(|f| f.file_path_hash == hash)
    }

    pub fn file_bytes_by_path(&self, path: &str) -> Result<Vec<u8>, AssetArchiveError> {
        if let Some(descriptor) = self.file_descriptor_by_path(path) {
            let mut bdt = fs::File::open(self.data_path.as_str())
                .map_err(AssetArchiveError::IO)?;

            bdt.seek(SeekFrom::Start(descriptor.file_offset))
                .map_err(AssetArchiveError::IO)?;

            let mut buffer = vec![0x0u8; descriptor.padded_file_size as usize];
            bdt.read_exact(&mut buffer)
                .map_err(AssetArchiveError::IO)?;

            // Decrypt the file in-place
            descriptor.decrypt_file(&mut buffer);

            // Determine appropriate truncation size to strip off any padding
            let truncation_size = if descriptor.file_size != 0 {
                descriptor.file_size
            } else {
                descriptor.padded_file_size
            };
            buffer.truncate(truncation_size as usize);

            Ok(buffer)
        } else {
            Err(AssetArchiveError::FileNotFound)
        }
    }
}
