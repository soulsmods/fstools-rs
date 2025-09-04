use std::{
    collections::HashMap,
    fmt::format,
    fs::File,
    io,
    io::{Cursor, Error, Read},
    ops::Range,
    path::{Path, PathBuf},
    slice,
};

use aes::{
    cipher::{consts::U16, generic_array::GenericArray, BlockDecrypt, BlockSizeUser, KeyInit},
    Aes128,
};
use fstools_formats::{
    bhd::Bhd,
    bnd4::{BND4Reader, BND4},
    dcx::DcxHeader,
};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameType {
    EldenRing,
    Nightreign,
}

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
    /// Read a generic dvdbnd dictionary text file's contents.
    /// Exposed so custom dictionaries can be used.
    pub fn dictionary(data_file_contents: &str) -> impl Iterator<Item = PathBuf> {
        data_file_contents
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(std::path::PathBuf::from)
            .collect::<Vec<PathBuf>>()
            .into_iter()
    }

    pub fn dictionary_from_game(game_type: GameType) -> impl Iterator<Item = PathBuf> {
        match game_type {
            GameType::EldenRing => {
                Self::dictionary(include_str!("../Data/EldenRingDictionary.txt"))
            }
            GameType::Nightreign => {
                Self::dictionary(include_str!("../Data/NightreignDictionary.txt"))
            }
        }
    }

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

    pub fn create_from_game(
        game_type: GameType,
        game_path: PathBuf,
        keys: impl ArchiveKeyProvider,
    ) -> Result<DvdBnd, io::Error> {
        let archives: &[PathBuf] = match &game_type {
            GameType::EldenRing => &[
                game_path.join("Data0"),
                game_path.join("Data1"),
                game_path.join("Data2"),
                game_path.join("Data3"),
                game_path.join("DLC"),
                game_path.join("sd/sd"),
                game_path.join("sd/sd_dlc02"),
            ],
            GameType::Nightreign => &[
                game_path.join("Data0"),
                game_path.join("Data1"),
                game_path.join("Data2"),
                game_path.join("Data3"),
                game_path.join("sd/sd"),
            ],
        };

        DvdBnd::create(archives, &keys)
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

    /// Read the bytes of a nested or non-nested file within the container
    pub fn read_file(
        &self,
        nested_bnd_names: &[String],
        name: &str,
    ) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
        let mut data = vec![];
        let cmp_string: String;

        if !nested_bnd_names.is_empty() {
            let dvdbnd_entry = nested_bnd_names.first().expect("No nested bnd entry");
            let (_, mut reader) = DcxHeader::read(self.open(dvdbnd_entry)?)?;
            reader.read_to_end(&mut data)?;

            if nested_bnd_names.len() > 1 {
                for n in nested_bnd_names[1..].iter() {
                    data = Self::read_nested_bnd(n, &data)?;
                }
            }

            data = Self::read_nested_bnd(name, &data)?;
            cmp_string = String::from("None");
        } else {
            let (dcx, mut reader) = DcxHeader::read(self.open(name)?)?;
            cmp_string = format(format_args!("{:?}", dcx.compression_parameters()));
            reader.read_to_end(&mut data)?;
        }

        Ok((cmp_string, data))
    }

    fn read_nested_bnd(
        nested_name: &str,
        parent_data: &Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let bnd = BND4::from_reader(&mut Cursor::new(parent_data))?;
        let nested_bnd_entry = bnd.files.iter().find(|entry| {
            *(Path::new(entry.path.as_str())
                .file_name()
                .expect("Nested bnd entry has no file name")
                .to_str()
                .expect("Nested bnd entry not found inside parent bnd"))
                == *nested_name
        });

        let mut bnd4_reader: BND4Reader = BND4Reader::new(bnd.data);
        let data_out = nested_bnd_entry.expect("No nested bnd entry").bytes(&mut bnd4_reader)?;

        Ok(data_out)
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
