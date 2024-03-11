use std::io::{Error, Read, Seek, SeekFrom};

use memmap2::MmapMut;

pub struct DvdBndEntryReader {
    mmap: MmapMut,
    position: usize,
}

impl DvdBndEntryReader {
    pub fn new(mmap: MmapMut) -> Self {
        Self { mmap, position: 0 }
    }
}

impl Read for DvdBndEntryReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut data = &self.mmap[self.position..];
        let read = data.read(buf)?;

        self.position += read;

        Ok(read)
    }
}

impl Seek for DvdBndEntryReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(start_offset) => Some(start_offset as usize),
            SeekFrom::End(end_offset) => self.mmap.len().checked_add_signed(end_offset as isize),
            SeekFrom::Current(offset) => self.position.checked_add_signed(offset as isize),
        }
        .ok_or(Error::other("invalid seek offset"))?;

        if new_pos < self.mmap.len() {
            self.position = new_pos;
            Ok(self.position as u64)
        } else {
            Err(Error::other("seek went out of bounds"))
        }
    }
}
