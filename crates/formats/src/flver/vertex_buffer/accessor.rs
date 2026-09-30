use std::{array, marker::PhantomData, mem::size_of};

use bytemuck::Pod;

/// Raw vertex attribute data accessor.
///
/// Each variant corresponds to a [`VertexFormat`](super::VertexFormat) and
/// yields the raw on-disk primitives .
#[derive(Debug)]
pub enum VertexAttributeAccessor<'a> {
    Float2(VertexAttributeIter<'a, f32, 2>),
    Float3(VertexAttributeIter<'a, f32, 3>),
    Float4(VertexAttributeIter<'a, f32, 4>),
    Uint8x4(VertexAttributeIter<'a, u8, 4>),
    Uint8x4WFirst(VertexAttributeIter<'a, u8, 4>),
    Uint16x2(VertexAttributeIter<'a, i16, 2>),
    Uint16x4(VertexAttributeIter<'a, i16, 4>),
    Uint16x4Biased(VertexAttributeIter<'a, u16, 4>),
}

pub struct VertexAttributeIter<'a, T: Pod, const L: usize> {
    buffer: &'a [u8],
    attribute_data_offset: usize,
    attribute_data_end: usize,
    vertex_size: usize,
    _value: PhantomData<T>,
}

impl<T: Pod, const L: usize> std::fmt::Debug for VertexAttributeIter<'_, T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexAttributeIter")
            .field("attribute_data_offset", &self.attribute_data_offset)
            .field("attribute_data_end", &self.attribute_data_end)
            .field("vertex_size", &self.vertex_size)
            .finish()
    }
}

// TODO: this doesn't support endian sensitive reading like the rest of the FLVER parser.
impl<'a, T: Pod, const L: usize> VertexAttributeIter<'a, T, L> {
    pub fn new(
        buffer: &'a [u8],
        vertex_size: usize,
        vertex_offset: usize,
    ) -> VertexAttributeIter<'a, T, L> {
        let attribute_data_offset = vertex_offset;
        let attribute_data_end = attribute_data_offset + size_of::<T>() * L;

        Self {
            buffer,
            attribute_data_offset,
            attribute_data_end,
            vertex_size,
            _value: PhantomData,
        }
    }
}

impl<T: Pod, const L: usize> ExactSizeIterator for VertexAttributeIter<'_, T, L> {}

impl<T: Pod, const L: usize> Iterator for VertexAttributeIter<'_, T, L> {
    type Item = [T; L];

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            return None;
        }

        let attribute_byte_data = &self.buffer[self.attribute_data_offset..self.attribute_data_end];
        let data: &[T] = bytemuck::cast_slice(attribute_byte_data);
        let output: [T; L] = array::from_fn(|index| data[index]);

        self.buffer = &self.buffer[self.vertex_size..];

        Some(output)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len() / self.vertex_size;
        (remaining, Some(remaining))
    }
}
