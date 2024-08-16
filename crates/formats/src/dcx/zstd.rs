use std::io::{self, Read};

/// Trivial wrapper around a [`zstd::Decoder<BufReader<R>>`].
pub struct ZstdDecoder<R: Read>(zstd::Decoder<'static, io::BufReader<R>>);

impl<R: Read> ZstdDecoder<R> {
    pub fn new(reader: R) -> io::Result<Self> {
        Ok(Self(zstd::Decoder::new(reader)?))
    }
}

impl<R: Read> Read for ZstdDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}
