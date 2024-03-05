use std::{ffi::c_void, io::{self, Cursor, Error, ErrorKind, Read, Result}, mem::size_of};

use byteorder::BE;
use oodle_sys::{
    OodleLZDecoder, OodleLZDecoder_Create, OodleLZDecoder_Destroy, OodleLZ_Compressor, OodleLZ_Compressor_OodleLZ_Compressor_Kraken, OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll, OodleLZ_Decompress, OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes, OODLELZ_FAILED
};
use zerocopy::U32;

// const DECODE_WINDOW_SIZE: i32 = 3 * 1024 * 1024;
// const DICTIONARY_SIZE: i32 = 2 * 1024 * 1024;
// const BUFFER_SIZE: i32 = (256 + 63) * 1024;
// const COMPRESSOR: OodleLZ_Compressor = OodleLZ_Compressor_OodleLZ_Compressor_Kraken;

pub struct DcxDecoderKraken<'a> {
    compressed: &'a [u8],
    uncompressed_size: U32<BE>,
    // decoder: *mut OodleLZDecoder,

    inner_cursor: Option<Cursor<Vec<u8>>>,
}

impl<'a> DcxDecoderKraken<'a> {
    pub fn from_buffer(
        buf: &'a [u8],
        uncompressed_size: U32<BE>,
    ) -> Self {
        // let raw_size = buf.len() as i64;
        // let decoder = unsafe {
        //     OodleLZDecoder_Create(
        //         COMPRESSOR,
        //         raw_size,
        //         0 as *mut c_void,
        //         -1,
        //     )
        // };

        Self {
            compressed: buf,
            uncompressed_size,
            // decoder,

            inner_cursor: None,
        }
    }
}

// impl<'a> Drop for DcxDecoderKraken<'a> {
//     fn drop(&mut self) {
//         unsafe { OodleLZDecoder_Destroy(self.decoder) }
//     }
// }

impl<'a> Read for DcxDecoderKraken<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Hack to just make it work for now
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

        // let mut dec_window = vec![0u8; DECODE_WINDOW_SIZE as usize];
        //
        // unsafe {
        //     let mut output = vec![0u8; size_of::<OodleLZ_DecodeSome_Out>()];
        //     OodleLZDecoder_DecodeSome(
        //         self.decoder,
        //         output.as_mut_ptr() as *mut OodleLZ_DecodeSome_Out,
        //         dec_window.as_mut_ptr() as *mut c_void, // dec_window
        //         DICTIONARY_SIZE as isize,               // dec_window_pos
        //         self.uncompressed_size.get() as isize,  // in_size
        //         DECODE_WINDOW_SIZE as isize - DICTIONARY_SIZE as isize, // dec_avail
        //         self.compressed.as_ptr() as *const c_void,
        //         self.compressed.len() as isize,
        //         OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes,
        //         OodleLZ_CheckCRC_OodleLZ_CheckCRC_No,
        //         OodleLZ_Verbosity_OodleLZ_Verbosity_None,
        //         OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll,
        //     );
        //
        //     dbg!(&dec_window);
        //     std::fs::write("./kraken-out.bin", dec_window).unwrap();
        //
        //     // let report = &*(output.as_ptr() as *const OodleLZ_DecodeSome_Out);
        //     Ok(0)
        // }
    }
}
