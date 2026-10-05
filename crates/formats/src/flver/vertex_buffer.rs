use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::{
    flver::{header::FlverHeaderPart, reader::VertexAttributeSemantic},
    io_ext::zerocopy::Padding,
};

pub mod accessor;
mod normalization;

#[derive(Debug, FromBytes, FromZeroes)]
#[allow(unused)]
#[repr(C, packed)]
pub struct VertexBuffer<O: ByteOrder> {
    pub buffer_index: U32<O>,
    pub layout_index: U32<O>,
    pub vertex_size: U32<O>,
    pub vertex_count: U32<O>,
    padding0: Padding<8>,
    pub buffer_length: U32<O>,
    pub buffer_offset: U32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for VertexBuffer<O> {}

#[derive(Debug, FromBytes, FromZeroes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct VertexBufferLayout<O: ByteOrder> {
    pub(crate) member_count: U32<O>,
    padding0: Padding<8>,
    pub(crate) member_offset: U32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for VertexBufferLayout<O> {}

#[derive(Debug, FromBytes, FromZeroes)]
#[repr(C, packed)]
#[allow(unused)]
pub struct VertexBufferAttribute<O: ByteOrder> {
    pub unk0: U32<O>,
    pub struct_offset: U32<O>,
    pub format_id: U32<O>,
    pub semantic_id: U32<O>,
    pub index: U32<O>,
}

impl<O: ByteOrder> VertexBufferAttribute<O> {
    #[allow(clippy::match_same_arms)]
    pub fn format(&self) -> Option<VertexFormat> {
        use VertexAttributeSemantic::*;
        use VertexFormat::*;

        let format = match (
            VertexAttributeSemantic::from(self.semantic_id.get()),
            self.format_id.get(),
        ) {
            (Position | UV, 0x02) => Float32x3,
            (Position | UV, 0x03) => Float32x4,
            (UV, 0x01) => Float32x2,
            (UV, 0x10 | 0x11 | 0x12 | 0x13 | 0x15) => Sscale16x2,
            (UV, 0x16) => Sscale16x4,
            (UV, 0x11A | 0x2E) => Sscale16x4,
            (Normal, 0x04) => Float32x4,
            (Normal, 0x10 | 0x11 | 0x13 | 0x2F) => Snorm8x4,
            (Normal, 0x12) => Snorm8x4, // soulstruct says unorm clamped to 127
            (Normal, 0x1A) => Snorm16x4,
            (Normal, 0x2E) => Snorm16x4, // soulstruct says unorm clamped to 127,
            (BoneWeights, 0x10) => Snorm8x4,
            (BoneWeights, 0x13) => Unorm8x4,
            (BoneWeights, 0x16 | 0x1A) => Snorm16x4,
            (BoneIndices, 0x11 | 0x24) => Uint8x4,
            (BoneIndices, 0x18) => Sint16x4,
            (Tangent, 0x10 | 0x11 | 0x13 | 0x2F) => Snorm8x4,
            _ => return None,
        };

        Some(format)
    }
}

pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
    Unorm8x4,
    Snorm8x4,
    Snorm16x4,
    Uint8x4,
    Sint16x4,
    Sscale16x2,
    Sscale16x4,
}

// UVs:
//
// 0x01: 2 floats
// 0x02: 3 floats (only 2 components used)
// 0x03: 2 floats, 2 floats
// 0x10, 0x11, 0x12, 0x13, 0x15: 2 signed shorts -> float (cast to float, take UV factor into
// account) 0x16: 4 signed shorts -> float (cast to float, take UV factor into account)
// 0x1A, 0x2E: 4 signed shorts -> float (cast to float, take UV factor into account) (only 2
// components used)
impl<O: ByteOrder> FlverHeaderPart for VertexBufferAttribute<O> {}
