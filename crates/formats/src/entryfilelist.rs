use std::{
    char,
    io::{self, BufReader, Read},
    mem::size_of,
};

use byteorder::{ReadBytesExt, LE};
use flate2::read::ZlibDecoder;
use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::io_ext::ReadWidestringError;

pub struct EntryFileListIterator<R: Read> {
    decompressor: BufReader<ZlibDecoder<R>>,
}

impl<R: Read> Iterator for EntryFileListIterator<R> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut string = String::new();

        loop {
            let c = self.decompressor.read_u16::<LE>().ok()?;
            // Read until NULL terminator
            if c == 0x0 {
                break;
            }

            string.push(char::from_u32(c as u32)?);
        }

        Some(string)
    }
}

#[derive(Debug, Error)]
pub enum EntryFileListError {
    #[error("Could not read string")]
    String(#[from] ReadWidestringError),

    #[error("Could not create reference to value")]
    UnalignedValue,

    #[error("Zlib error")]
    Zlib,

    #[error("Io error")]
    Io(#[from] io::Error),
}

#[allow(unused)]
pub struct EntryFileListContainer;

impl EntryFileListContainer {
    pub fn read<R: Read>(
        mut reader: R,
    ) -> Result<(ContainerHeader, EntryFileListIterator<R>), EntryFileListError> {
        let mut header_data = [0u8; size_of::<ContainerHeader>()];
        reader.read_exact(&mut header_data)?;

        let header =
            ContainerHeader::read_from(&header_data).ok_or(EntryFileListError::UnalignedValue)?;
        let mut decompressor = BufReader::new(ZlibDecoder::new(reader));

        let _unk0 = decompressor.read_u32::<LE>()?;
        let unk1_count = decompressor.read_u32::<LE>()?;
        let unk2_count = decompressor.read_u32::<LE>()?;
        let _unkc = decompressor.read_u32::<LE>()?;

        for _ in 0..unk1_count {
            let _ = Unk1::parse(&mut decompressor)?;
        }
        // FIXME: decompressor.seek_until_alignment(0x10)?;

        for _ in 0..unk2_count {
            let _ = decompressor.read_u64::<LE>()?;
        }
        // FIXME: decompressor.seek_until_alignment(0x10)?;

        let _unk = decompressor.read_u16::<LE>()?;
        let iter = EntryFileListIterator { decompressor };

        Ok((header, iter))
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct ContainerHeader {
    magic: [u8; 4],

    unk04: U32<LE>,

    compressed_size: U32<LE>,

    decompressed_size: U32<LE>,
}

#[derive(Debug)]
pub struct Unk1 {
    pub step: u16,
    pub index: u16,
}

impl Unk1 {
    pub fn parse<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Self {
            step: reader.read_u16::<LE>()?,
            index: reader.read_u16::<LE>()?,
        })
    }
}
