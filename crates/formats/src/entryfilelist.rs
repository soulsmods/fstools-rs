use std::{
    char,
    io::{self, ErrorKind, Read}, marker::PhantomData,
};

use byteorder::{ReadBytesExt, LE};
use flate2::read::ZlibDecoder;
use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, Ref, U32};

use crate::io_ext::ReadWidestringError;

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
pub struct EntryFileListContainer<'a, T: EntryFileListSection> {
    decoder: ZlibDecoder<&'a [u8]>,
    header: &'a ContainerHeader,
    unk1_count: usize,
    unk2_count: usize,
    marker: PhantomData<T>,
}

pub struct Unk1Section;
pub struct Unk2Section;
pub struct StringsSection;

pub trait EntryFileListSection {}
impl EntryFileListSection for Unk1Section {}
impl EntryFileListSection for Unk2Section {}
impl EntryFileListSection for StringsSection {}

impl<'a> EntryFileListContainer<'a, Unk1Section> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, EntryFileListError> {
        let (header, compressed) = Ref::<_, ContainerHeader>::new_from_prefix(bytes)
            .ok_or(EntryFileListError::UnalignedValue)?;

        let mut decoder = ZlibDecoder::new(compressed);

        let _unk0 = decoder.read_u32::<LE>()?;
        let unk1_count = decoder.read_u32::<LE>()? as usize;
        let unk2_count = decoder.read_u32::<LE>()? as usize;
        let _unkc = decoder.read_u32::<LE>()?;

        Ok(Self {
            decoder,
            header: header.into_ref(),
            unk1_count,
            unk2_count,
            marker: PhantomData,
        })
    }

    pub fn unk1s(mut self) -> Result<(
        Vec<Unk1>,
        EntryFileListContainer<'a, Unk2Section>,
    ), EntryFileListError> {
        let elements = (0..self.unk1_count)
            .map(|_| Unk1::parse(&mut self.decoder))
            .collect::<Result<_, _>>()?;

        // TODO: reach alignment
        todo!();

        Ok((elements, EntryFileListContainer {
            decoder: self.decoder,
            header: self.header,
            unk1_count: self.unk1_count,
            unk2_count: self.unk2_count,
            marker: PhantomData,
        }))
    }
}

impl<'a> EntryFileListContainer<'a, Unk2Section> {
    pub fn unk2s(mut self) -> Result<(
        Vec<u64>,
        EntryFileListContainer<'a, StringsSection>,
    ), EntryFileListError> {
        let elements = (0..self.unk2_count)
            .map(|_| self.decoder.read_u64::<LE>())
            .collect::<Result<_, _>>()?;

        // TODO: reach alignment

        Ok((elements, EntryFileListContainer {
            decoder: self.decoder,
            header: self.header,
            unk1_count: self.unk1_count,
            unk2_count: self.unk2_count,
            marker: PhantomData,
        }))
    }
}

impl<'a> EntryFileListContainer<'a, StringsSection> {
    pub fn strings(mut self) -> Result<Vec<String>, EntryFileListError> {
        let elements = (0..self.unk2_count)
            .map(|_| Self::read_string(&mut self.decoder))
            .collect::<Result<_, _>>()?;

        Ok(elements)
    }

    fn read_string<R: Read>(mut reader: R) -> Result<String, io::Error> {
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

impl<'a, T: EntryFileListSection> std::fmt::Debug for EntryFileListContainer<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryFileList")
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
