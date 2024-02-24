use std::collections::HashMap;
use std::io::{Error, Read};
use std::ops::Range;
use std::{fs::File, path::Path};

use format::bhd::Bhd;
use memmap2::{Advice, Mmap, MmapOptions};
use thiserror::Error;

mod bnd;
mod key_provider;
mod name;
mod reader;

pub use self::{
    key_provider::ArchiveKeyProvider,
    key_provider::FileKeyProvider,
    name::Name,
    reader::VfsEntryReader,
    bnd::BndMountHost,
    bnd::undo_container_compression,
};

#[derive(Debug, Error)]
pub enum VfsOpenError {
    #[error("Mmap operation failed: {0}")]
    Mmap(#[from] Error),

    #[error("Entry was not found")]
    NotFound,
}

/// A read-only virtual filesystem layered over the BHD/BDT archives of a FROMSOFTWARE game.
pub struct Vfs {
    archives: Vec<Mmap>,
    entries: HashMap<Name, VfsFileEntry>,
    mount_host: BndMountHost,
}

impl Vfs {
    fn load_archive<P: AsRef<Path>>(
        path: P,
        key_provider: &impl ArchiveKeyProvider,
    ) -> Result<(Mmap, Bhd), Error> {
        let path = path.as_ref();
        let bhd_file = File::open(path.with_extension("bhd"))?;
        let bdt_file = File::open(path.with_extension("bdt"))?;
        let data = unsafe { MmapOptions::new().map_copy_read_only(&bdt_file)? };
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(Error::other("invalid archive path given"))?;

        let key = key_provider.get_key(name)?;
        let bhd = Bhd::read(bhd_file, key)?;

        Ok((data, bhd))
    }

    /// Create a virtual filesystem from the archive files (BHD or BDT) pointed to by [archive_paths].
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
                let (data, bhd) = Self::load_archive(path, key_provider)?;

                archives.push(data);
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

        Ok(Vfs { archives, entries, mount_host: Default::default(), })
    }

    /// Open a reader to the file identified by [name].
    pub fn open<N: Into<Name>>(&self, name: N) -> Result<VfsEntryReader, VfsOpenError> {
        match self.entries.get(&name.into()) {
            Some(entry) => {
                let mmap = &self.archives[entry.archive];
                let offset = entry.file_offset as usize;
                let size = entry.file_size_with_padding as usize;

                mmap.advise_range(Advice::Sequential, offset, size)?;

                Ok(VfsEntryReader::new(&mmap[offset..offset + size], entry))
            }
            None => Err(VfsOpenError::NotFound),
        }
    }

    /// Attaches a bnd4 to the mount host
    pub fn mount<N: Into<Name>>(&mut self, name: N) -> Result<(), VfsOpenError> {
        let name = name.into();

        let mut reader = self.open(name.clone())?;
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();

        self.mount_host.mount(
            name,
            buffer.as_slice(),
        ).unwrap();

        Ok(())
    }

    pub fn open_from_mounts(&self, name: &str) -> Result<&[u8], VfsOpenError> {
        self.mount_host.bytes_by_file_name(name)
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
