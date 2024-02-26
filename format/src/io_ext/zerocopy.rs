use zerocopy::{FromBytes, FromZeroes};

#[derive(FromZeroes, FromBytes)]
#[repr(packed)]
pub struct Padding<const N: usize>([u8; N]);
