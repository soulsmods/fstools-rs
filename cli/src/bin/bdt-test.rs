use std::fs;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use clap::Parser;
use format::bhd::hash_path;
use format::bhd::FileDescriptor;
use std::io::Read;
use format::bhd::{BHDError, BHD};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    archive: String,
    #[arg(long)]
    key: String,
    #[arg(long)]
    dictionary: String,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut key_file = std::fs::File::open(args.key)?;
    let mut key = Vec::new();
    key_file.read_to_end(&mut key)?;

    let archive = GameArchive::new(&args.archive, &key)
        .expect("Could not open game archive");

    let dictionary_file = std::fs::read_to_string(args.dictionary)?;
    for file in dictionary_file.lines() {
        if file.starts_with("#") {
            continue;
        }

        let bytes = archive.file_bytes_by_path(file)
            .expect("Could not retrieve file from index");

        match bytes {
            Some(bytes) => {
                let output_path = std::path::PathBuf::from(format!("./test/{}", file));
                fs::create_dir_all(output_path.parent().unwrap())?;
                fs::write(output_path, bytes)?;
            },
            None => println!("Could not find entry {} in archive", file),
        }
    }

    Ok(())
}

#[derive(Debug)]
enum GameArchiveError {
    IO(io::Error),
    BHD(BHDError),
}

struct GameArchive {
    header: BHD,
    data_path: String,
}

impl GameArchive {
    pub fn new(path: &str, key: &[u8]) -> Result<Self, GameArchiveError> {
        let mut header_file = fs::File::open(format!("{}.bhd", path))
            .map_err(GameArchiveError::IO)?;

        let header = BHD::from_reader_with_key(
            &mut header_file,
            key,
        ).map_err(GameArchiveError::BHD)?;

        Ok(Self {
            header,
            data_path: format!("{}.bdt", path),
        })
    }

    pub fn file_descriptor_by_path(&self, path: &str) -> Option<&FileDescriptor> {
        self.file_descriptor_by_hash(hash_path(path))
    }

    fn file_descriptor_by_hash(&self, hash: u64) -> Option<&FileDescriptor> {
        self.header.buckets.iter()
            .flat_map(|b| b.files.as_slice())
            .find(|f| f.file_path_hash == hash)
    }

    pub fn file_bytes_by_path(&self, path: &str) -> Result<Option<Vec<u8>>, GameArchiveError> {
        if let Some(descriptor) = dbg!(self.file_descriptor_by_path(path)) {
            let mut bdt = fs::File::open(self.data_path.as_str())
                .map_err(GameArchiveError::IO)?;

            bdt.seek(SeekFrom::Start(descriptor.file_offset))
                .map_err(GameArchiveError::IO)?;

            let mut buffer = vec![0x0u8; descriptor.padded_file_size as usize];
            bdt.read_exact(&mut buffer)
                .map_err(GameArchiveError::IO)?;

            // Decrypt the file in-place
            descriptor.decrypt_file(&mut buffer);

            // Determine appropriate truncation size to strip off any padding
            let truncation_size = if descriptor.file_size != 0 {
                descriptor.file_size
            } else {
                descriptor.padded_file_size
            };
            buffer.truncate(truncation_size as usize);

            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }
}
