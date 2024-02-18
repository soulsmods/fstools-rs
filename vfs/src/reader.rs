use std::cmp::min;
use std::io::{Cursor, Error, Read, Write};
use std::ops::Range;

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::{Aes128, Block};

use crate::VfsFileEntry;

pub struct VfsEntryReader<'a> {
    /// Block cipher used for the optionally encrypted data blocks.
    cipher: Aes128,

    /// Underlying raw data stream.
    data: &'a [u8],

    /// Current reader position into [data].
    data_pos: usize,

    /// The last read encrypted block, if any.
    encrypted_block: Block,

    /// The current offset into the last read encrypted block.
    encrypted_block_offset: Option<usize>,

    /// The current encrypted data range being processed.
    encrypted_data_range_index: usize,

    /// The ranges of data in this file that are encrypted.
    encrypted_data_ranges: &'a [Range<u64>],

    /// The size of the file including any padding from encryption.
    encrypted_file_size: usize,
}

pub enum VfsEntryPartKind {
    Ciphertext,
    Plaintext,
}

impl<'a> VfsEntryReader<'a> {
    pub fn new(data: &'a [u8], entry: &'a VfsFileEntry) -> Self {
        Self {
            cipher: Aes128::new(&GenericArray::from(entry.aes_key)),
            data,
            data_pos: 0,
            encrypted_block: Block::default(),
            encrypted_block_offset: None,
            encrypted_data_range_index: 0,
            encrypted_data_ranges: &entry.aes_ranges,
            encrypted_file_size: entry.file_size_with_padding as usize,
        }
    }

    fn read_plaintext(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        buf.copy_from_slice(&self.data[self.data_pos..self.data_pos + buf.len()]);
        self.data_pos += buf.len();

        Ok(buf.len())
    }

    fn read_ciphertext(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let block_len = self.encrypted_block.len();
        let blocks = (buf.len() + block_len - 1) / block_len;
        let mut writer = Cursor::new(buf);
        let mut written = 0;

        if let Some(offset) = self.encrypted_block_offset.take() {
            written += writer.write(&self.encrypted_block[offset..])?;
        }

        for _ in 0..blocks {
            let block_data = &self.data[self.data_pos..self.data_pos + block_len];
            self.encrypted_block.copy_from_slice(block_data);
            self.cipher.decrypt_block(&mut self.encrypted_block);

            // SAFETY: `write` on a cursor cannot fail.
            let block_written = unsafe { writer.write(&self.encrypted_block).unwrap_unchecked() };
            written += block_written;

            self.data_pos += block_len;

            // Couldn't write the complete block, continue on the next read.
            if block_written < block_len {
                self.encrypted_block_offset = Some(block_written);
                break;
            }
        }

        Ok(written)
    }
}

impl<'a> Read for VfsEntryReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let requested = buf.len();
        let remaining = self.encrypted_file_size - self.data_pos;
        let readable = min(requested, remaining);

        let mut read = 0;

        while read < readable {
            let pos = self.data_pos;
            let range = self
                .encrypted_data_ranges
                .get(self.encrypted_data_range_index);

            let (part_type, part_size) = match range {
                Some(range) if range.contains(&(pos as u64)) => {
                    (VfsEntryPartKind::Ciphertext, range.end as usize - pos)
                }
                Some(range) => (VfsEntryPartKind::Plaintext, range.start as usize - pos),
                None => (VfsEntryPartKind::Plaintext, remaining),
            };

            let out_offset = read;
            let out_capacity = min(out_offset + part_size, buf.len());
            let out = &mut buf[out_offset..out_capacity];

            let part_read = match part_type {
                VfsEntryPartKind::Ciphertext => self.read_ciphertext(out)?,
                VfsEntryPartKind::Plaintext => self.read_plaintext(out)?,
            };

            read += part_read;

            if part_read == part_size && matches!(part_type, VfsEntryPartKind::Ciphertext) {
                self.encrypted_data_range_index += 1;
            } else if part_read < part_size {
                // Buffer not large enough, resume on next read.
                break;
            }
        }

        Ok(read)
    }
}

#[cfg(test)]
mod test {
    fn create_test_data() {}

    #[test]
    fn decodes() {}
}
