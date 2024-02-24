use byteorder::ReadBytesExt;
use std::io;
use std::io::{ErrorKind, Read};

pub trait ReadFormatsExt {
    fn read_bool(&mut self) -> std::io::Result<bool>;
    fn read_magic<const LENGTH: usize>(&mut self, expected: &[u8; LENGTH]) -> std::io::Result<()>;

    fn read_padding(&mut self, length: usize) -> std::io::Result<()>;
}

impl<R: Read> ReadFormatsExt for R {
    fn read_bool(&mut self) -> std::io::Result<bool> {
        Ok(self.read_u8()? == 1)
    }

    #[inline]
    fn read_magic<const LENGTH: usize>(&mut self, expected: &[u8; LENGTH]) -> std::io::Result<()> {
        let mut buffer = [0u8; LENGTH];
        self.read_exact(&mut buffer)?;

        if &buffer == expected {
            Ok(())
        } else {
            Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "expected {:?} ({:#x?}), found {:?} ({:#x?})",
                    String::from_utf8_lossy(expected),
                    expected,
                    String::from_utf8_lossy(&buffer),
                    &buffer
                ),
            ))
        }
    }

    #[cfg(not(debug_assertions))]
    fn read_padding(&mut self, length: usize) -> std::io::Result<()> {
        let mut taken = self.take(length as u64);
        std::io::copy(&mut taken, &mut std::io::sink())?;
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn read_padding(&mut self, length: usize) -> std::io::Result<()> {
        for _ in 0..length {
            let padding = self.read_u8()?;

            if padding != 0 {
                dbg!("Expected padding bytes, found non-zero value: {}", padding);
            }
        }

        Ok(())
    }
}
