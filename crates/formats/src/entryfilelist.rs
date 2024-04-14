use std::{
    char,
    io::{self, ErrorKind, Read},
    marker::PhantomData,
};

use byteorder::{ReadBytesExt, LE};
use flate2::read::ZlibDecoder;
use thiserror::Error;
use zerocopy::{FromBytes, FromZeroes, Ref, U32};

use crate::io_ext::{ReadFormatsExt, ReadWidestringError};

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

    #[error("Iterator not exhausted")]
    IteratorNotExhausted,
}

#[allow(unused)]
pub struct EntryFileList<'a> {
    decoder: ZlibDecoder<&'a [u8]>,
    container_header: &'a ContainerHeader,
    header: EntryFileListHeader,
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(packed)]
#[allow(unused)]
struct ContainerHeader {
    magic: [u8; 4],
    _unk04: U32<LE>,
    compressed_size: U32<LE>,
    decompressed_size: U32<LE>,
}

#[derive(Debug)]
pub struct EntryFileListHeader {
    pub unk1_count: usize,
    pub unk2_count: usize,
}

impl<'a> EntryFileList<'a> {
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
            container_header: header.into_ref(),
            header: EntryFileListHeader {
                unk1_count,
                unk2_count,
            },
        })
    }
}

impl<'a> std::fmt::Debug for EntryFileList<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryFileList")
            .field("unk04", &self.container_header._unk04.get())
            .field(
                "compressed_size",
                &self.container_header.compressed_size.get(),
            )
            .field(
                "decompressed_size",
                &self.container_header.decompressed_size.get(),
            )
            .field("unk1_count", &self.header.unk1_count)
            .field("unk2_count", &self.header.unk2_count)
            .finish()
    }
}

impl<'a> IntoIterator for EntryFileList<'a> {
    type Item = Result<Unk1, EntryFileListError>;
    type IntoIter = SectionIter<'a, Unk1Section>;

    fn into_iter(self) -> Self::IntoIter {
        SectionIter {
            decoder: self.decoder,
            entry_count: self.header.unk1_count,
            header: self.header,
            entries_read: 0,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct SectionIter<'a, TSection> {
    decoder: ZlibDecoder<&'a [u8]>,
    header: EntryFileListHeader,
    entry_count: usize,
    entries_read: usize,
    _marker: PhantomData<TSection>,
}

pub trait EntryFileListSection {
    type Element;

    fn read_element(r: impl Read) -> Result<Self::Element, EntryFileListError>
    where
        Self: Sized;
}

impl<'a, TSection> Iterator for SectionIter<'a, TSection>
where
    TSection: EntryFileListSection,
{
    type Item = Result<TSection::Element, EntryFileListError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries_read < self.entry_count {
            let result = (self.entries_read..self.entry_count)
                .next()
                .map(|_| TSection::read_element(&mut self.decoder));

            self.entries_read += 1;

            result
        } else {
            None
        }
    }
}

pub struct Unk1Section;

impl EntryFileListSection for Unk1Section {
    type Element = Unk1;

    fn read_element(mut r: impl Read) -> Result<Self::Element, EntryFileListError> {
        Ok(Unk1 {
            step: r.read_u16::<LE>()?,
            index: r.read_u16::<LE>()?,
        })
    }
}

#[derive(Debug)]
pub struct Unk1 {
    pub step: u16,
    pub index: u16,
}

impl<'a> SectionIter<'a, Unk1Section> {
    // TODO: can probably deduplicate some code between here and SectionIter<'a, Unk2Section>
    pub fn next_section(mut self) -> Result<SectionIter<'a, Unk2Section>, EntryFileListError> {
        if self.entries_read != self.entry_count {
            return Err(EntryFileListError::IteratorNotExhausted);
        }

        self.decoder.read_padding(offset_for_alignment(
            self.decoder.total_out() as usize,
            0x10,
        ))?;

        Ok(SectionIter {
            decoder: self.decoder,
            entry_count: self.header.unk2_count,
            header: self.header,
            entries_read: 0,
            _marker: PhantomData,
        })
    }
}

impl<'a> SectionIter<'a, Unk2Section> {
    pub fn next_section(mut self) -> Result<SectionIter<'a, UnkStringSection>, EntryFileListError> {
        if self.entries_read != self.entry_count {
            return Err(EntryFileListError::IteratorNotExhausted);
        }


        self.decoder.read_padding(offset_for_alignment(
            self.decoder.total_out() as usize,
            0x10,
        ))?;

        // This is seems to be some value since the unk2 count excludes this
        // if it were a string. It's always 0x0000 though so :shrug:
        self.decoder.read_padding(2)?;

        Ok(SectionIter {
            decoder: self.decoder,
            entry_count: self.header.unk2_count,
            header: self.header,
            entries_read: 0,
            _marker: PhantomData,
        })
    }
}

pub struct Unk2Section;

impl EntryFileListSection for Unk2Section {
    type Element = u64;

    fn read_element(mut r: impl Read) -> Result<Self::Element, EntryFileListError> {
        Ok(r.read_u64::<LE>()?)
    }
}

pub struct UnkStringSection;

impl EntryFileListSection for UnkStringSection {
    type Element = String;

    fn read_element(mut r: impl Read) -> Result<Self::Element, EntryFileListError> {
        let mut string = String::new();

        loop {
            let c = r.read_u16::<LE>()?;
            if c == 0x0 {
                break;
            }

            string.push(char::from_u32(c as u32).ok_or(io::Error::from(ErrorKind::InvalidData))?);
        }

        Ok(string)
    }
}

fn offset_for_alignment(current: usize, align: usize) -> usize {
    let offset = current % align;

    if offset == 0 {
        0
    } else {
        align - offset
    }
}
