use std::io::{Cursor, Error, ErrorKind, Read, Result};

use byteorder::BE;
use oodle_sys::{
    OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll, OodleLZ_Decompress, OODLELZ_FAILED,
};
use zerocopy::U32;

pub struct DcxDecoderKraken<'a> {
    compressed: &'a [u8],
    uncompressed_size: U32<BE>,
    inner_cursor: Option<Cursor<Vec<u8>>>,
}

impl<'a> DcxDecoderKraken<'a> {
    pub fn from_buffer(buf: &'a [u8], uncompressed_size: U32<BE>) -> Self {
        Self {
            compressed: buf,
            uncompressed_size,
            inner_cursor: None,
        }
    }
}
impl<'a> Read for DcxDecoderKraken<'a> {
    // TODO: implement somewhat incremental reading by working with oodle's
    // blocks per example in docs.
    // It currently just decompresses the entire input one go and then
    // operates a Cursor wrapping the decompressed bytes.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.inner_cursor.is_none() {
            let mut inner_buffer = vec![0u8; self.uncompressed_size.get() as usize];

            let result = unsafe {
                OodleLZ_Decompress(
                    self.compressed.as_ptr() as *const _,
                    self.compressed.len() as isize,
                    inner_buffer.as_mut_ptr() as *mut _,
                    inner_buffer.len() as isize,
                    oodle_sys::OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes,
                    0,
                    0,
                    std::ptr::null_mut(),
                    0,
                    None,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0,
                    OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll,
                ) as usize
            };

            if result == OODLELZ_FAILED as usize {
                return Err(Error::from(ErrorKind::Other));
            }

            self.inner_cursor = Some(Cursor::new(inner_buffer));
        }

        self.inner_cursor.as_mut().unwrap().read(buf)
    }
}
