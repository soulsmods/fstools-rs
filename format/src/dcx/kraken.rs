use std::{
    cmp::min,
    io::{BufRead, BufReader, Error, Read, Result, Take},
    ptr::null_mut,
};

use oodle_sys::{
    OodleLZDecoder, OodleLZDecoder_Create, OodleLZDecoder_DecodeSome,
    OodleLZ_CheckCRC_OodleLZ_CheckCRC_Yes, OodleLZ_Compressor_OodleLZ_Compressor_Kraken,
    OodleLZ_DecodeSome_Out, OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_Unthreaded,
    OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes, OodleLZ_Verbosity_OodleLZ_Verbosity_Lots,
    OODLELZ_BLOCK_LEN,
};

pub struct DcxDecoderKraken<R: Read> {
    reader: BufReader<Take<R>>,

    /// The total size of the raw data expected to be read from the underlying stream.
    uncompressed_size: u32,

    /// The Oodle decoder instance created for this buffer.
    decoder: *mut OodleLZDecoder,

    /// A sliding window of bytes decoded by the compressor, large enough to keep the past block in
    /// memory while the next block is decoded.
    sliding_window: Box<[u8]>,

    /// The decoders position into the sliding window.
    sliding_window_pos: usize,

    /// The number of bytes that the consuming reader is lagging behind the decoder.
    sliding_window_lag: usize,
}

impl<R: Read> DcxDecoderKraken<R> {
    // TODO: fix vfs reader so it isn't producing padding
    pub fn new(reader: Take<R>, uncompressed_size: u32) -> Self {
        let compressor = OodleLZ_Compressor_OodleLZ_Compressor_Kraken;
        let decoder = unsafe {
            OodleLZDecoder_Create(compressor, uncompressed_size as i64, null_mut(), 0isize)
        };

        if decoder.is_null() {
            panic!("return error here: failed to create decoder, check oodle error");
        }

        let sliding_window = Box::new([0u8; (OODLELZ_BLOCK_LEN * 2) as usize]);

        Self {
            decoder,
            reader: BufReader::with_capacity(OODLELZ_BLOCK_LEN as usize, reader),
            sliding_window,
            sliding_window_pos: 0,
            sliding_window_lag: 0,
            uncompressed_size,
        }
    }
}

impl<R: Read> Read for DcxDecoderKraken<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut total_written = 0usize;
        while total_written < buf.len() {
            let wpos = self.sliding_window_pos;

            // Check if there's data to be written from the sliding window first
            if self.sliding_window_lag > 0 {
                let bytes_to_copy = min(self.sliding_window_lag, buf.len() - total_written);
                let start = self.sliding_window_pos - self.sliding_window_lag;
                let end = start + bytes_to_copy;

                let src = &self.sliding_window[start..end];
                let dest = &mut buf[total_written..total_written + bytes_to_copy];

                dest.copy_from_slice(src);

                self.sliding_window_lag -= bytes_to_copy;
                total_written += bytes_to_copy;

                continue;
            }

            // Read and decode new data
            let input_data = self.reader.fill_buf()?;
            if input_data.is_empty() {
                break; // EOF reached
            }

            let mut out: OodleLZ_DecodeSome_Out = unsafe { std::mem::zeroed() };
            let result = unsafe {
                // EXTREMELY unlikely, however unsound otherwise.
                let input_data_len = isize::try_from(input_data.len()).unwrap_or(isize::MAX);

                // SAFETY:
                // - Signedness conversions of offsets are all valid, given that
                //   `sliding_window.len() <= i32::MAX` and `self.uncompressed_size < isize::MAX`.
                // - Consumed `input_data_len` is caped at i32::MAX
                OodleLZDecoder_DecodeSome(
                    self.decoder,
                    &mut out as *mut _,
                    self.sliding_window.as_mut_ptr() as *mut _,
                    wpos as isize,
                    self.uncompressed_size as _,
                    (self.sliding_window.len() - wpos) as isize,
                    input_data.as_ptr() as *const _,
                    input_data_len,
                    OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_Yes,
                    OodleLZ_CheckCRC_OodleLZ_CheckCRC_Yes,
                    OodleLZ_Verbosity_OodleLZ_Verbosity_Lots,
                    OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_Unthreaded,
                )
            };

            if result == 0 {
                return Err(Error::other("Oodle decoder failed"));
            }

            let decoded_bytes = out.decodedCount as usize;
            let consumed_bytes = out.compBufUsed as usize;

            self.reader.consume(consumed_bytes);

            let bytes_to_copy = min(decoded_bytes, buf.len() - total_written);
            let dest = &mut buf[total_written..total_written + bytes_to_copy];
            let src = &self.sliding_window[wpos..wpos + bytes_to_copy];

            dest.copy_from_slice(src);

            self.sliding_window_pos += decoded_bytes;
            self.sliding_window_lag = decoded_bytes - bytes_to_copy;
            total_written += bytes_to_copy;

            // Manage sliding window
            if self.sliding_window_pos >= self.sliding_window.len() {
                self.sliding_window.rotate_left(self.sliding_window_pos);
                self.sliding_window_pos = 0;
            }
        }

        Ok(total_written)
    }
}
