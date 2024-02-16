use std::fs;
use std::collections;
use clap::Parser;
use format::bhd::hash_path;
use util::AssetArchive;
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

    let archive = AssetArchive::new(&args.archive, &key)
        .expect("Could not open game archive");

    let dictionary_contents = std::fs::read_to_string(args.dictionary)?;
    let dictionary = dictionary_contents.lines()
        .filter(|f| !f.starts_with("#"))
        .map(|f| (hash_path(f), f))
        .collect::<collections::HashMap<u64, &str>>();

    for file in archive.files().into_iter() {
        match dictionary.get(&file.file_path_hash) {
            Some(p) => println!("{}", p),
            None => println!("Unknown file: {}", file.file_path_hash),
        }
    }

    Ok(())
}
