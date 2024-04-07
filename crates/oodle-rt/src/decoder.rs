use std::ptr::NonNull;

use crate::{
    ffi::{OodleLZDecoder, OodleLZ_CheckCRC, OodleLZ_FuzzSafe},
    DecodeSome_Out, DecodeThreadPhase, Oodle, Verbosity,
};

pub struct OodleDecoder {
    oodle: Oodle,
    ptr: NonNull<OodleLZDecoder>,
    uncompressed_size: usize,
}

impl OodleDecoder {
    pub fn new(oodle: Oodle, ptr: NonNull<OodleLZDecoder>, uncompressed_size: usize) -> Self {
        Self {
            oodle,
            ptr,
            uncompressed_size,
        }
    }

    pub fn decode_some(
        &self,
        decode_buffer: &mut [u8],
        decode_buffer_pos: usize,
        compressed_data: &[u8],
    ) -> Option<DecodeSome_Out> {
        let mut output = DecodeSome_Out::default();
        let decode_buffer_avail = decode_buffer.len() - decode_buffer_pos;
        let input_data_len = isize::try_from(compressed_data.len()).unwrap_or(isize::MAX);
        let func = self
            .oodle
            .oodle_lz_decoder_decode_some
            .expect("missing symbol");

        // SAFETY: Only valid slices and pointers to valid FFI objects are passed to the
        // decompressor.
        let result = unsafe {
            (func)(
                self.ptr.as_ptr(),
                &mut output as *mut _,
                decode_buffer.as_mut_ptr().cast(),
                decode_buffer_pos as isize,
                self.uncompressed_size as isize,
                decode_buffer_avail as isize,
                compressed_data.as_ptr().cast(),
                input_data_len,
                OodleLZ_FuzzSafe::OodleLZ_FuzzSafe_No,
                OodleLZ_CheckCRC::OodleLZ_CheckCRC_Yes,
                Verbosity::OodleLZ_Verbosity_None,
                DecodeThreadPhase::OodleLZ_Decode_Unthreaded,
            )
        };

        match result {
            0 => None,
            _ => Some(output),
        }
    }
}

impl Default for DecodeSome_Out {
    fn default() -> Self {
        // SAFETY: 0 is a valid bit pattern for all fields.
        unsafe { std::mem::zeroed() }
    }
}

impl Drop for OodleDecoder {
    fn drop(&mut self) {
        let api = Oodle::current().expect("must be initialized to get an OodleDecoder");

        // Safety: guaranteed to be a valid decoder pointer.
        unsafe { (api.oodle_lz_decoder_destroy.expect("missing symbol"))(self.ptr.as_mut()) }
    }
}
