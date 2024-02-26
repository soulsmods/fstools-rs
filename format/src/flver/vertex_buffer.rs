use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::io_ext::zerocopy::Padding;

#[derive(FromBytes, FromZeroes)]
#[allow(unused)]
pub struct VertexBuffer<O: ByteOrder> {
    pub buffer_index: U32<O>,
    pub layout_index: U32<O>,
    pub vertex_size: U32<O>,
    pub vertex_count: U32<O>,
    padding0: Padding<8>,
    pub buffer_length: U32<O>,
    pub buffer_offset: U32<O>,
}

#[derive(FromBytes, FromZeroes)]
#[allow(unused)]
pub struct VertexBufferLayout<O: ByteOrder> {
    member_count: U32<O>,
    padding0: Padding<8>,
    member_offset: U32<O>,
}

#[derive(FromBytes, FromZeroes)]
#[repr(packed)]
#[allow(unused)]
pub struct VertexBufferLayoutMember<O: ByteOrder> {
    pub unk0: U32<O>,
    pub struct_offset: U32<O>,
    pub format_id: U32<O>,
    pub semantic_id: U32<O>,
    pub index: U32<O>,
}
