use fstools_formats::bnd4::BND4;

use crate::archive::IntoArchive;

impl IntoArchive for BND4 {
    fn files(&self) -> impl Iterator<Item = (String, Vec<u8>)> {
        self.files.iter().map(|file| {
            let start = file.data_offset as usize;
            let size = file.compressed_size as usize;

            (file.path.clone(), self.data[start..start + size].to_vec())
        })
    }
}
