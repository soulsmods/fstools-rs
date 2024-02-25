use std::{io::Read, ops::Deref};

use ::zerocopy::{FromBytes, FromZeroes, Ref, F32, U32};
use byteorder::{ByteOrder, BE, LE};

use crate::{
    flver::dummy::{FlverDummy, FlverDummyData},
    io_ext::{zerocopy::Padding, ReadFormatsExt},
};

pub mod accessor;
mod dummy;
mod mesh;
pub mod reader;

pub enum Flver<'a> {
    LittleEndian(FlverInner<'a, LE>),
    BigEndian(FlverInner<'a, BE>),
}

impl<'a> Deref for Flver<'a> {
    type Target = dyn FlverHeader;

    fn deref(&self) -> &Self::Target {
        match self {
            Flver::LittleEndian(inner) => inner.header,
            Flver::BigEndian(inner) => inner.header,
        }
    }
}

impl<'a> Flver<'a> {
    pub fn dummy(&'a self, index: usize) -> &'a dyn FlverDummy {
        match self {
            Flver::LittleEndian(inner) => inner.dummy(index),
            Flver::BigEndian(inner) => inner.dummy(index),
        }
    }
}

impl<'a> Flver<'a> {
    fn parse<O: ByteOrder + 'static>(data: &'a [u8]) -> Option<FlverInner<'a, O>> {
        let (header_ref, dummy_bytes) = Ref::<&'a [u8], FlverHeaderData<O>>::new_from_prefix(data)?;
        let header: &'a FlverHeaderData<O> = header_ref.into_ref();
        let dummy_count = header.dummy_count.get() as usize;
        let (dummys, _next) = FlverDummyData::<O>::slice_from_prefix(dummy_bytes, dummy_count)?;

        Some(FlverInner { header, dummys })
    }

    pub fn from(data: &'a [u8]) -> Result<Self, std::io::Error> {
        let mut header = &data[..8];
        header.read_magic(b"FLVER\0")?;

        let mut endianness = vec![0x0u8; 2];
        header.read_exact(&mut endianness)?;

        let is_little_endian = endianness == [0x4c, 0x00];
        let flver = if is_little_endian {
            Self::parse(data).map(Flver::LittleEndian)
        } else {
            Self::parse(data).map(Flver::BigEndian)
        };

        flver.ok_or(std::io::Error::other("data buffer was not unaligned"))
    }
}

pub struct FlverInner<'a, O: ByteOrder> {
    header: &'a FlverHeaderData<O>,
    dummys: &'a [FlverDummyData<O>],
}

pub trait FlverData {
    fn dummy(&self, index: usize) -> &dyn FlverDummy;
}

impl<'a, O: ByteOrder> FlverData for FlverInner<'a, O> {
    fn dummy(&self, index: usize) -> &dyn FlverDummy {
        &self.dummys[index]
    }
}

pub type FlverHeaderLE = FlverHeaderData<LE>;
pub type FlverHeaderBE = FlverHeaderData<BE>;

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
pub struct FlverHeaderData<O: ByteOrder> {
    padding0: Padding<8>,
    pub(crate) version: U32<O>,
    pub(crate) data_offset: U32<O>,
    pub(crate) data_length: U32<O>,
    pub(crate) dummy_count: U32<O>,
    pub(crate) material_count: U32<O>,
    pub(crate) bone_count: U32<O>,
    pub(crate) mesh_count: U32<O>,
    pub(crate) vertex_buffer_count: U32<O>,
    pub(crate) bounding_box_min: [F32<O>; 3],
    pub(crate) bounding_box_max: [F32<O>; 3],
    pub(crate) face_count: U32<O>,
    pub(crate) total_face_count: U32<O>,
    pub(crate) vertex_index_size: u8,
    pub(crate) unicode: u8,
    pub(crate) _unk4a: u8,
    pub(crate) _unk4b: u8,
    pub(crate) _unk4c: U32<O>,
    pub(crate) face_set_count: U32<O>,
    pub(crate) buffer_layout_count: U32<O>,
    pub(crate) texture_count: U32<O>,
    pub(crate) _unk5c: u8,
    pub(crate) _unk5d: u8,
    _padding1: Padding<8>,
    pub(crate) _unk68: U32<O>,
    _padding2: Padding<20>,
}

pub trait FlverHeader {
    fn version(&self) -> u32;
}

impl<E: ByteOrder> FlverHeader for FlverHeaderData<E> {
    fn version(&self) -> u32 {
        self.version.get()
    }
}
