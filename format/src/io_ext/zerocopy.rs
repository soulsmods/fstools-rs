use zerocopy::{FromBytes, FromZeroes};

#[derive(FromZeroes, FromBytes)]
#[repr(C)]
pub struct Padding<const N: usize>([u8; N]);
