use std::{marker::PhantomData, mem::size_of};

use bytemuck::Pod;

pub enum VertexAttributeAccessor<'a> {
    Float2(VertexAttributeIter<'a, [f32; 2]>),
    Float3(VertexAttributeIter<'a, [f32; 3]>),
    Float4(VertexAttributeIter<'a, [f32; 4]>),
    Byte4A(VertexAttributeIter<'a, [u8; 4]>),
    Byte4B(VertexAttributeIter<'a, [u8; 4]>),
    Short2ToFloat2(VertexAttributeIter<'a, [u16; 2]>),
    Byte4C(VertexAttributeIter<'a, [u8; 4]>),
    UV(VertexAttributeIter<'a, [f32; 2]>),
    // TODO: get the last 2 components of this
    UVPair(VertexAttributeIter<'a, [f32; 2]>),
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

// TODO: this doesn't support endian sensitive reading like the rest of the FLVER parser.
impl<'a, T: Pod> VertexAttributeIter<'a, T> {
    pub fn new(
        buffer: &'a [u8],
        vertex_size: usize,
        vertex_offset: usize,
    ) -> VertexAttributeIter<'a, T> {
        let attribute_data_offset = vertex_offset;
        let attribute_data_end = attribute_data_offset + size_of::<T>();

        Self {
            buffer,
            attribute_data_offset,
            attribute_data_end,
            vertex_size,
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
        let data: &T = bytemuck::from_bytes(attribute_byte_data);

        self.buffer = &self.buffer[self.vertex_size..];

        Some(*data)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len() / self.vertex_size;
        (remaining, Some(remaining))
    }
}
