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
/// Normalize a signed value to a range of [0,1] with N possible values.
pub struct SNorm<T: Into<f32> + Copy, const N: usize> {
    _value: PhantomData<T>,
}

impl<T: Into<f32> + Copy, const N: usize> VertexAttributeNormalization for SNorm<T, N> {
    type Input = T;
    type Output = f32;

    fn normalize(input: &Self::Input) -> Self::Output {
        ((*input).into() - N as f32) / N as f32
    }
}

/// Normalize an unsigned value to a range of [0,1] with N possible values.
pub struct UNorm<T: Into<f32> + Copy, const N: usize> {
    _value: PhantomData<T>,
}

impl<T: Into<f32> + Copy, const N: usize> VertexAttributeNormalization for UNorm<T, N> {
    type Input = T;
    type Output = f32;

    fn normalize(input: &Self::Input) -> Self::Output {
        (*input).into() / N as f32
    }
}
