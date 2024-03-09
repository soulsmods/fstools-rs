use std::io::Read;

use memmap2::MmapMut;

pub struct VfsEntryReader {
    mmap: MmapMut,
    position: usize,
}

impl VfsEntryReader {
    pub fn new(mmap: MmapMut) -> Self {
        Self { mmap, position: 0 }
    }
}

impl Read for VfsEntryReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut data = &self.mmap[self.position..];
        let read = data.read(buf)?;

        self.position += read;

        Ok(read)
    }
}
