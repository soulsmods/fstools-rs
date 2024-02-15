use std::fs;
use clap::Parser;
use rayon::iter::ParallelIterator;
use indicatif::ParallelProgressIterator;
use rayon::iter::IntoParallelRefIterator;
use util::GameArchive;
use std::io::Read;

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

    let dictionary = std::fs::read_to_string(args.dictionary)?;
    let lines = dictionary.lines()
        .map(|l| std::path::PathBuf::from(l))
        .collect::<Vec<_>>();

    lines.par_iter()
        .progress()
        .filter(|l| !l.starts_with("#"))
        .for_each(|l| {
            let path = l.to_str().unwrap();
            let bytes = archive.file_bytes_by_path(path)
                .expect("Could not retrieve file from index");

            match bytes {
                Some(bytes) => {
                    let output_path = std::path::PathBuf::from(format!("./test/{}", path));
                    let directory = output_path.parent().unwrap();
                    fs::create_dir_all(directory).unwrap();
                    fs::write(output_path, bytes).unwrap();
                },
                None => {},
            }
        });

    Ok(())
}
