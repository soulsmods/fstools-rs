use std::fmt::{Debug, Formatter};

use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[derive(AsBytes, FromZeroes, FromBytes, Copy, Clone)]
#[repr(C, packed)]
pub struct Padding<const N: usize>([u8; N]);

impl<const N: usize> Debug for Padding<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Padding").field("length", &N).finish()
    }
}
