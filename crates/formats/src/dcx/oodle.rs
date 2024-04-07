use std::{
    cmp::min,
    io::{Error, Read, Result},
};

use fstools_oodle_rt::{decoder::OodleDecoder, Compressor, Oodle, OODLELZ_BLOCK_LEN};

// SAFETY: `OodleLZDecoder` pointer is safe to use across several threads.
unsafe impl<R: Read> Sync for OodleReader<R> {}

// SAFETY: See above.
unsafe impl<R: Read> Send for OodleReader<R> {}

pub struct OodleReader<R: Read> {
    reader: R,

    /// The Oodle decoder instance created for this buffer.
    decoder: OodleDecoder,

    /// A sliding window of bytes decoded by the compressor, large enough to keep the past block in
    /// memory while the next block is decoded.
    decode_buffer: Box<[u8]>,

    /// The decoders position into the sliding window.
    decode_buffer_writer_pos: usize,

    /// The number of bytes that the consuming reader is lagging behind the decoder.
    decode_buffer_reader_lag: usize,

    /// Oodle requires at least [`OODLELZ_BLOCK_LEN`] bytes available in the input buffer, which
    /// the read buffer might not fit. Instead, we buffer to this intermediate buffer and treat
    /// it as a sliding window to ensure there are always `OODLELZ_BLOCK_LEN` bytes available
    /// to read.
    io_buffer: Box<[u8]>,

    /// The number of bytes available to read from [`io_buffer`], ending at [`io_buffer_pos`].
    io_buffer_writer_pos: usize,

    /// Current position within the IO buffer.
    io_buffer_reader_pos: usize,
}

impl<R: Read> OodleReader<R> {
    pub fn new(reader: R, uncompressed_size: u32) -> Option<Self> {
        let oodle = Oodle::current()?;
        let decoder = oodle.create_decoder(
            Compressor::OodleLZ_Compressor_Kraken,
            uncompressed_size as usize,
        )?;
        let decode_buffer = vec![0u8; 3 * 1024 * 1024].into_boxed_slice();
        let io_buffer = vec![0u8; OODLELZ_BLOCK_LEN as usize * 2].into_boxed_slice();

        Some(Self {
            // SAFETY: Pointer is validated to be non-null above.
            decoder,
            reader,
            decode_buffer,
            decode_buffer_writer_pos: 0,
            decode_buffer_reader_lag: 0,
            io_buffer,
            io_buffer_reader_pos: 0,
            io_buffer_writer_pos: 0,
        })
    }
}

impl<R: Read> Read for OodleReader<R> {
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

            let out = self
                .decoder
                .decode_some(&mut self.decode_buffer, wpos, data)
                .ok_or(Error::other("Oodle decoder failed"))?;
            // SAFETY: OodleLZ_DecodeSome_out is zero initialised by default.

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
