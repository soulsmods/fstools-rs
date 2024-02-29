use std::marker::PhantomData;

pub trait VertexAttributeNormalization {
    type Input;
    type Output;

    fn normalize(input: &Self::Input) -> Self::Output;
}

pub struct NoNormalization<T> {
    _phantom: PhantomData<T>,
}

impl<T: Copy> VertexAttributeNormalization for NoNormalization<T> {
    type Input = T;
    type Output = T;

    fn normalize(input: &Self::Input) -> Self::Output {
        *input
    }
}

/// Normalize an unsigned 4-bit value to a range of [0, 1] with 128 possible values.
pub struct UNorm4;

impl VertexAttributeNormalization for UNorm4 {
    type Input = u8;
    type Output = f32;

    fn normalize(input: &Self::Input) -> Self::Output {
        *input as f32 / 127.0
    }
}

/// Normalize an unsigned 8-bit value to a range of [0,1] with 256 possible values.
pub struct UNorm8;

impl VertexAttributeNormalization for UNorm8 {
    type Input = u8;
    type Output = f32;

    fn normalize(input: &Self::Input) -> Self::Output {
        *input as f32 / 255.0
    }
}
