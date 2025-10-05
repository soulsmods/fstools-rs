use typed_path::Utf8WindowsPathBuf;

pub struct WindowsPath<'a> {
    inner: &'a typed_path::WindowsPath,
}

impl<'a> WindowsPath<'a> {
    pub fn new<S: AsRef<[u8]> + ?Sized>(data: &'a S) -> Self {
        Self {
            inner: typed_path::WindowsPath::new(data),
        }
    }

    pub fn to_path_buf(&self) -> Utf8WindowsPathBuf {
        let wchars: Vec<u16> = self
            .inner
            .as_bytes()
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Utf8WindowsPathBuf::from(String::from_utf16_lossy(&wchars))
    }
}
