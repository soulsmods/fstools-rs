use std::{
    char,
    io::{self, Cursor, ErrorKind, Read, Seek},
};

use byteorder::{ReadBytesExt, LE};
use flate2::read::ZlibDecoder;
use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, Ref, U32};

use crate::io_ext::{ReadWidestringError, SeekExt};

#[derive(Debug, Error)]
pub enum EntryfilelistError {
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
pub struct EntryfilelistContainer<'a> {
    bytes: &'a [u8],

    header: &'a ContainerHeader,

    compressed: &'a [u8],
}

impl<'a> EntryfilelistContainer<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, EntryfilelistError> {
        let (header, compressed) = Ref::<_, ContainerHeader>::new_from_prefix(bytes)
            .ok_or(EntryfilelistError::UnalignedValue)?;

        Ok(Self {
            bytes,
            header: header.into_ref(),
            compressed,
        })
    }

    fn hint_size(&self) -> usize {
        self.header.decompressed_size.get() as usize
    }

    pub fn decompress(&self) -> Result<Entryfilelist, EntryfilelistError> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.hint_size());
        let mut decoder = ZlibDecoder::new(self.compressed);

        decoder
            .read_to_end(&mut buf)
            .map_err(|_| EntryfilelistError::Zlib)?;

        Ok(Entryfilelist::parse(Cursor::new(buf))?)
    }
}

impl<'a> std::fmt::Debug for EntryfilelistContainer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entryfilelist")
            .field("unk04", &self.header.unk04.get())
            .field("compressed_size", &self.header.compressed_size.get())
            .field("decompressed_size", &self.header.decompressed_size.get())
            .finish()
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
#[allow(unused)]
pub struct Entryfilelist {
    pub unk1: Vec<Unk1>,
    pub unk2: Vec<u64>,
    pub strings: Vec<String>,
}

impl Entryfilelist {
    pub fn parse<R: Read + Seek>(mut reader: R) -> Result<Self, io::Error> {
        let _unk0 = reader.read_u32::<LE>()?;
        let unk1_count = reader.read_u32::<LE>()?;
        let unk2_count = reader.read_u32::<LE>()?;
        let _unkc = reader.read_u32::<LE>()?;

        let unk1 = (0..unk1_count)
            .map(|_| Unk1::parse(&mut reader))
            .collect::<Result<_, _>>()?;
        reader.seek_until_alignment(0x10)?;

        let unk2 = (0..unk2_count)
            .map(|_| reader.read_u64::<LE>())
            .collect::<Result<_, _>>()?;
        reader.seek_until_alignment(0x10)?;

        let _unk = reader.read_u16::<LE>()?;

        let strings = (0..unk2_count)
            .map(|_| Self::read_string(&mut reader))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            unk1,
            unk2,
            strings,
        })
    }

    pub fn read_string<R: Read>(mut reader: R) -> Result<String, io::Error> {
        let mut string = String::new();

        loop {
            // We always know read the right amount of strings so we
            // shouldn't encounter EOF
            let c = reader.read_u16::<LE>()?;
            // Read until NULL terminator
            if c == 0x0 {
                break;
            }

            string.push(char::from_u32(c as u32).ok_or(io::Error::from(ErrorKind::InvalidData))?);
        }

        Ok(string)
    }
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct Header {
    unk0: U32<LE>,
    count_unk1: U32<LE>,
    count_unk2: U32<LE>,
    unkc: U32<LE>,
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
