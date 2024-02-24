use std::{marker::PhantomData, mem::size_of};

use bytemuck::Pod;

use crate::flver::{FLVERBufferLayoutMember, VertexBuffer};

pub enum VertexAttributeAccessor<'a> {
    Float2(VertexAttributeIter<'a, [f32; 2]>),
    Float3(VertexAttributeIter<'a, [f32; 3]>),
    Float4(VertexAttributeIter<'a, [f32; 4]>),
    Byte4A(VertexAttributeIter<'a, [u8; 4]>),
    Byte4B(VertexAttributeIter<'a, [u8; 4]>),
    Short2ToFloat2(VertexAttributeIter<'a, [u16; 2]>),
    Byte4C(VertexAttributeIter<'a, [u8; 4]>),
    UV(VertexAttributeIter<'a, [f32; 2]>),
    UVPair(VertexAttributeIter<'a, [f32; 4]>),
    Short4ToFloat4A(VertexAttributeIter<'a, [u16; 4]>),
    Short4ToFloat4B(VertexAttributeIter<'a, [u16; 4]>),
}

pub struct VertexAttributeIter<'a, T: Pod> {
    buffer: &'a [u8],
    attribute_data_offset: usize,
    attribute_data_end: usize,
    vertex_size: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: Pod> VertexAttributeIter<'a, T> {
    pub fn new(
        data: &'a [u8],
        buffer_info: &VertexBuffer,
        member: &FLVERBufferLayoutMember,
    ) -> VertexAttributeIter<'a, T> {
        let buffer_offset = buffer_info.buffer_offset as usize;
        let buffer_length = buffer_info.buffer_length as usize;
        let buffer = &data[buffer_offset..buffer_offset + buffer_length];
        let attribute_data_offset = member.struct_offset as usize;
        let attribute_data_end = attribute_data_offset + size_of::<T>();

        Self {
            buffer,
            attribute_data_offset,
            attribute_data_end,
            vertex_size: buffer_info.vertex_size as usize,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T: Pod> ExactSizeIterator for VertexAttributeIter<'a, T> {}
impl<'a, T: Pod> Iterator for VertexAttributeIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            return None;
        }

        let attribute_byte_data = &self.buffer[self.attribute_data_offset..self.attribute_data_end];
        let data: &[T] = bytemuck::cast_slice(attribute_byte_data);

        self.buffer = &self.buffer[self.vertex_size..];

        Some(data[0])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len() / self.vertex_size;
        (remaining, Some(remaining))
    }
}
