use std::{
    cmp::min,
    io::{Error, Read, Result},
    ptr::{null_mut, NonNull},
};

use oodle_sys::{
    OodleLZDecoder, OodleLZDecoder_Create, OodleLZDecoder_DecodeSome, OodleLZDecoder_Destroy,
    OodleLZ_CheckCRC_OodleLZ_CheckCRC_Yes, OodleLZ_Compressor_OodleLZ_Compressor_Kraken,
    OodleLZ_DecodeSome_Out, OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll,
    OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_No,
    OodleLZ_Verbosity_OodleLZ_Verbosity_None, OODLELZ_BLOCK_LEN,
};

// SAFETY: `OodleLZDecoder` pointer is safe to use across several threads.
unsafe impl<R: Read> Sync for OodleDecoder<R> {}

// SAFETY: See above.
unsafe impl<R: Read> Send for OodleDecoder<R> {}

pub struct OodleDecoder<R: Read> {
    reader: R,

    /// The total size of the raw data expected to be read from the underlying stream.
    uncompressed_size: u32,

    /// The Oodle decoder instance created for this buffer.
    decoder: NonNull<OodleLZDecoder>,

    /// A sliding window of bytes decoded by the compressor, large enough to keep the past block in
    /// memory while the next block is decoded.
    decode_buffer: Box<[u8]>,

    /// The decoders position into the sliding window.
    decode_buffer_writer_pos: usize,

    /// The number of bytes that the consuming reader is lagging behind the decoder.
    decode_buffer_reader_lag: usize,

    /// Oodle requires at least [OODLELZ_BLOCK_LEN] bytes available in the input buffer, which the
    /// read buffer might not fit. Instead, we buffer to this intermediate buffer and treat it as a
    /// sliding window to ensure there are always OODLELZ_BLOCK_LEN bytes available to read.
    io_buffer: Box<[u8]>,

    /// The number of bytes available to read from [io_buffer], ending at [io_buffer_pos].
    io_buffer_writer_pos: usize,

    /// Current position within the IO buffer.
    io_buffer_reader_pos: usize,
}

impl<R: Read> OodleDecoder<R> {
    pub fn new(reader: R, uncompressed_size: u32) -> Option<Self> {
        let compressor = OodleLZ_Compressor_OodleLZ_Compressor_Kraken;

        // SAFETY: Calling OodleLZDecoder_Create with any `uncompressed_size` is safe.
        let decoder = unsafe {
            OodleLZDecoder_Create(compressor, uncompressed_size as i64, null_mut(), 0isize)
        };

        // Oodle was unable to create a decoder for this compressor
        if decoder.is_null() {
            return None;
        }

        let decode_buffer = vec![0u8; 3 * 1024 * 1024].into_boxed_slice();
        let io_buffer = vec![0u8; OODLELZ_BLOCK_LEN as usize * 2].into_boxed_slice();

        Some(Self {
            // SAFETY: Pointer is validated to be non-null above.
            decoder: unsafe { NonNull::new_unchecked(decoder) },
            reader,
            decode_buffer,
            decode_buffer_writer_pos: 0,
            decode_buffer_reader_lag: 0,
            io_buffer,
            io_buffer_reader_pos: 0,
            io_buffer_writer_pos: 0,
            uncompressed_size,
        })
    }
}

impl<R: Read> Read for OodleDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let dictionary_size = 2 * 1024 * 1024;
        let mut total_written = 0usize;

        while total_written < buf.len() {
            let wpos = self.decode_buffer_writer_pos;

            // Check if there's data to be written from the sliding window first
            if self.decode_buffer_reader_lag > 0 {
                let bytes_to_copy = min(self.decode_buffer_reader_lag, buf.len() - total_written);
                let start = wpos - self.decode_buffer_reader_lag;
                let end = start + bytes_to_copy;

                let src = &self.decode_buffer[start..end];
                let dest = &mut buf[total_written..total_written + bytes_to_copy];

                dest.copy_from_slice(src);

                self.decode_buffer_reader_lag -= bytes_to_copy;
                total_written += bytes_to_copy;

                continue;
            }

            self.io_buffer_writer_pos += self
                .reader
                .read(&mut self.io_buffer[self.io_buffer_writer_pos..])?;

            let data = &self.io_buffer[self.io_buffer_reader_pos..self.io_buffer_writer_pos];
            // Read and decode new data
            if data.is_empty() {
                break; // EOF reached
            }

            // SAFETY: OodleLZ_DecodeSome_out is zero initialised by default.
            let mut out: OodleLZ_DecodeSome_Out = unsafe { std::mem::zeroed() };

            // SAFETY:
            // - Signedness conversions of offsets are all valid, given that `sliding_window.len()
            //   <= i32::MAX` and `self.uncompressed_size < isize::MAX`.
            // - Consumed `input_data_len` is caped at i32::MAX
            let result = unsafe {
                // EXTREMELY unlikely, however unsound otherwise.
                let input_data_len = isize::try_from(data.len()).unwrap_or(isize::MAX);
                let decode_buffer_avail = self.decode_buffer.len() - wpos;

                OodleLZDecoder_DecodeSome(
                    self.decoder.as_ptr(),
                    &mut out as *mut _,
                    self.decode_buffer.as_mut_ptr().cast(),
                    wpos as isize,
                    self.uncompressed_size as isize,
                    decode_buffer_avail as isize,
                    data.as_ptr().cast(),
                    input_data_len,
                    OodleLZ_FuzzSafe_OodleLZ_FuzzSafe_No,
                    OodleLZ_CheckCRC_OodleLZ_CheckCRC_Yes,
                    OodleLZ_Verbosity_OodleLZ_Verbosity_None,
                    OodleLZ_Decode_ThreadPhase_OodleLZ_Decode_ThreadPhaseAll,
                )
            };

            if result == 0 {
                return Err(Error::other("Oodle decoder failed"));
            }

            let decoded_bytes = out.decodedCount as usize;
            let consumed_bytes = out.compBufUsed as usize;

            self.io_buffer_reader_pos += consumed_bytes;

            if decoded_bytes > 0 {
                let bytes_to_copy = min(decoded_bytes, buf.len() - total_written);
                let dest = &mut buf[total_written..total_written + bytes_to_copy];
                let src = &self.decode_buffer[wpos..wpos + bytes_to_copy];

                dest.copy_from_slice(src);

                self.decode_buffer_writer_pos += decoded_bytes;
                self.decode_buffer_reader_lag = decoded_bytes - bytes_to_copy;
                total_written += bytes_to_copy;
            } else {
                // Nothing more to decode.
                if out.curQuantumCompLen == 0 {
                    return Ok(total_written);
                }

                let remaining = self.io_buffer_writer_pos - self.io_buffer_reader_pos;

                self.io_buffer.rotate_left(self.io_buffer_reader_pos);
                self.io_buffer_reader_pos = 0;
                self.io_buffer_writer_pos = remaining;
            }

            // Manage sliding window
            if self.decode_buffer_writer_pos + OODLELZ_BLOCK_LEN as usize > self.decode_buffer.len()
            {
                self.decode_buffer.copy_within(
                    self.decode_buffer_writer_pos - dictionary_size..self.decode_buffer_writer_pos,
                    0,
                );

                self.decode_buffer_writer_pos = dictionary_size;
            }
        }

        Ok(total_written)
    }
}

impl<R: Read> Drop for OodleDecoder<R> {
    fn drop(&mut self) {
        // SAFETY: Guaranteed to be a valid pointer to a Decoder by [OodleDecoder::new].
        unsafe { OodleLZDecoder_Destroy(self.decoder.as_ptr()) }
    }
}
