use std::{collections::HashMap, fs::File, io::Error, ops::Range, path::Path, slice};

use aes::{
    cipher::{consts::U16, generic_array::GenericArray, BlockDecrypt, BlockSizeUser, KeyInit},
    Aes128,
};
use fstools_formats::bhd::Bhd;
use memmap2::MmapOptions;
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};
use thiserror::Error;

pub use self::{
    key_provider::{ArchiveKeyProvider, FileKeyProvider},
    name::Name,
    reader::DvdBndEntryReader,
};

mod key_provider;
mod name;
mod reader;

#[derive(Debug, Error)]
pub enum DvdBndEntryError {
    #[error("Corrupt entry header")]
    CorruptEntry,

    #[error("Entry was not found")]
    NotFound,

    #[error("Failed to map file data")]
    UnableToMap(#[from] Error),
}

/// A read-only virtual filesystem layered over the BHD/BDT archives of a FROMSOFTWARE game.
pub struct DvdBnd {
    archives: Vec<File>,
    entries: HashMap<Name, VfsFileEntry>,
}

impl DvdBnd {
    fn load_archive<P: AsRef<Path>>(
        path: P,
        key_provider: &impl ArchiveKeyProvider,
    ) -> Result<(File, Bhd), Error> {
        let path = path.as_ref();
        let bhd_file = File::open(path.with_extension("bhd"))?;
        let bdt_file = File::open(path.with_extension("bdt"))?;
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(Error::other("invalid archive path given"))?;

        let key = key_provider.get_key(name)?;
        let bhd = Bhd::read(bhd_file, key)?;

        Ok((bdt_file, bhd))
    }

    /// Create a virtual filesystem from the archive files (BHD or BDT) pointed to by
    /// [`archive_paths`].
    pub fn create<P: AsRef<Path>, K: ArchiveKeyProvider>(
        archive_paths: impl IntoIterator<Item = P>,
        key_provider: &K,
    ) -> Result<Self, Error> {
        let mut archives = Vec::new();
        let mut entries = HashMap::new();

        archive_paths
            .into_iter()
            .enumerate()
            .try_for_each(|(index, path)| {
                let path = path.as_ref();
                let (mmap, bhd) = Self::load_archive(path, key_provider)?;

                archives.push(mmap);
                entries.extend(bhd.toc.into_iter().map(|entry| {
                    (
                        Name(entry.hash),
                        VfsFileEntry {
                            archive: index,
                            file_size: entry.size,
                            file_size_with_padding: entry.padded_size,
                            file_offset: entry.offset,
                            aes_key: entry.aes_key,
                            aes_ranges: entry
                                .encrypted_ranges
                                .into_iter()
                                .filter_map(|range| match range {
                                    (-1, -1) => None,
                                    (start, end) if start == end => None,
                                    (start, end) => Some(start as u64..end as u64),
                                })
                                .collect(),
                        },
                    )
                }));

                Ok::<_, Error>(())
            })?;

        Ok(DvdBnd { archives, entries })
    }

    /// Open a reader to the file identified by [name].
    pub fn open<N: Into<Name>>(&self, name: N) -> Result<DvdBndEntryReader, DvdBndEntryError> {
        match self.entries.get(&name.into()) {
            Some(entry) => {
                let archive_file = &self.archives[entry.archive];
                let offset = entry.file_offset as usize;
                let encrypted_size = entry.file_size_with_padding as usize;

                // SAFETY: no safety guarantees here. File could be modified while we read from it.
                let mut mmap = unsafe {
                    MmapOptions::new()
                        .offset(offset as u64)
                        .len(encrypted_size)
                        .map_copy(archive_file)?
                };

                let data_ptr = mmap.as_mut_ptr();
                let data_cipher = Aes128::new(&GenericArray::from(entry.aes_key));
                let encrypted_blocks: Result<Vec<&mut [GenericArray<u8, U16>]>, _> = entry
                    .aes_ranges
                    .iter()
                    .map(|range| {
                        let size = (range.end - range.start) as usize;

                        if range.start >= mmap.len() as u64 || range.end > mmap.len() as u64 {
                            return Err(DvdBndEntryError::CorruptEntry);
                        }

                        let num_blocks = size / Aes128::block_size();

                        // SAFETY: We check the offset added to `data_ptr` is within the bounds of a
                        // valid pointer.
                        let blocks: &mut [GenericArray<u8, U16>] = unsafe {
                            slice::from_raw_parts_mut(
                                data_ptr.add(range.start as usize).cast(),
                                num_blocks,
                            )
                        };

                        Ok(blocks)
                    })
                    .collect();

                encrypted_blocks?
                    .into_iter()
                    .par_bridge()
                    .for_each(|blocks| {
                        data_cipher.decrypt_blocks(blocks);
                    });

                #[cfg(unix)]
                let _ = mmap.advise(memmap2::Advice::Sequential);

                // DCXes dont have an unpadded size set
                let effective_file_size = if entry.file_size != 0 {
                    entry.file_size
                } else {
                    entry.file_size_with_padding
                } as usize;

                Ok(DvdBndEntryReader::new(
                    mmap.make_read_only()?,
                    effective_file_size,
                ))
            }
            None => Err(DvdBndEntryError::NotFound),
        }
    }
}

#[derive(Debug)]
pub struct VfsFileEntry {
    archive: usize,
    #[allow(unused)]
    file_size: u32,
    file_size_with_padding: u32,
    file_offset: u64,
    aes_key: [u8; 16],
    aes_ranges: Vec<Range<u64>>,
}
