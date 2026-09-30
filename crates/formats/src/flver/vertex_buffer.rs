use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::{
    flver::{header::FlverHeaderPart, reader::VertexAttributeSemantic},
    io_ext::zerocopy::Padding,
};

pub mod accessor;

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

/// The raw data layout of a vertex attribute as stored on disk.
///
/// This enum represents **only** the byte-level encoding, the specific interpretation is up
/// to the shader that processes these attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
    Uint8x4,
    Uint8x4WFirst,
    Sint16x2,
    Sint16x4,
    Uint16x4,
    Uint16x4Biased,
}

impl<O: ByteOrder> VertexBufferAttribute<O> {
    /// Determine the raw data format for this attribute based on the
    /// combination of semantic and format-id.
    #[allow(clippy::match_same_arms)]
    pub fn format(&self) -> Option<VertexFormat> {
        use VertexAttributeSemantic::*;
        use VertexFormat::*;

        let semantic = VertexAttributeSemantic::from(self.semantic_id.get());
        let fmt = self.format_id.get();

        let result = match (semantic, fmt) {
            (Position, 0x02) => Float32x3,
            (Position, 0x03) => Float32x4,
            (Normal, 0x02) => Float32x3,
            (Normal, 0x03 | 0x04) => Float32x4,
            (Normal, 0x10 | 0x11 | 0x13 | 0x2F) => Uint8x4,
            (Normal, 0x12) => Uint8x4WFirst,
            (Normal, 0x1A) => Sint16x4,
            (Normal, 0x2E) => Uint16x4Biased,
            (Tangent, 0x03 | 0x04) => Float32x4,
            (Tangent, 0x10 | 0x11 | 0x13 | 0x2F) => Uint8x4,
            (Tangent, 0x1A) => Sint16x4,
            (UV, 0x01) => Float32x2,
            (UV, 0x02  | 0x03) => Float32x3,
            (UV, 0x10 | 0x11 | 0x12 | 0x13 | 0x15) => Sint16x2,
            (UV, 0x16 | 0x1A | 0x2E) => Sint16x4,
            (BoneIndices, 0x11 | 0x13 | 0x24 | 0x2F) => Uint8x4,
            (BoneIndices, 0x18) => Uint16x4,
            (BoneWeights, 0x10 | 0x13) => Uint8x4,
            (BoneWeights, 0x16 | 0x1A) => Sint16x4,
            (Bitangent, 0x10 | 0x11 | 0x13 | 0x2F) => Uint8x4,
            (VertexColor, 0x03 | 0x04) => Float32x4,
            (VertexColor, 0x10 | 0x13) => Uint8x4,

            _ => return None,
        };

        Some(result)
    }
}

impl<O: ByteOrder> FlverHeaderPart for VertexBufferAttribute<O> {}
