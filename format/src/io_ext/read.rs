use std::io::{ErrorKind, Read};

use byteorder::{ByteOrder, ReadBytesExt};

pub trait ReadFormatsExt {
    fn read_bool(&mut self) -> std::io::Result<bool>;
    fn read_magic<const LENGTH: usize>(&mut self, expected: &[u8; LENGTH]) -> std::io::Result<()>;
    fn read_utf16<BO: ByteOrder>(&mut self) -> std::io::Result<String>;

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

    fn read_utf16<BO: ByteOrder>(&mut self) -> std::io::Result<String> {
        let mut buffer = Vec::new();

        loop {
            let current = self.read_u16::<BO>()?;
            if current != 0x0 {
                buffer.push(current);
            } else {
                break;
            }
        }

        String::from_utf16(buffer.as_slice())
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, e.to_string()))
    }

    #[cfg(not(feature = "strict-padding"))]
    fn read_padding(&mut self, length: usize) -> std::io::Result<()> {
        let mut taken = self.take(length as u64);
        std::io::copy(&mut taken, &mut std::io::sink())?;
        Ok(())
    }

    #[cfg(feature = "strict-padding")]
    #[inline(always)]
    fn read_padding(&mut self, length: usize) -> std::io::Result<()> {
        for _ in 0..length {
            let padding = self.read_u8()?;

            if padding != 0 {
                return Err(std::io::Error::other(
                    "Expecting padding bytes, found non-zero value",
                ));
            }
        }

        Ok(())
    }
}
