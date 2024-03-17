use std::io::{self, ErrorKind, Seek, SeekFrom};

pub trait SeekExt: Seek {
    fn seek_until_alignment(&mut self, alignment: usize) -> io::Result<usize>;
}

impl<T: Seek> SeekExt for T {
    fn seek_until_alignment(&mut self, alignment: usize) -> io::Result<usize> {
        let current = self.stream_position()? as usize;
        let difference = if current % alignment == 0 {
            0
        } else {
            alignment - current % alignment
        };

        let difference_offset =
            i64::try_from(difference).map_err(|_| io::Error::from(ErrorKind::InvalidData))?;
        self.seek(SeekFrom::Current(difference_offset))?;

        Ok(difference)
    }
}
