use std::{error::Error, path::PathBuf};

use clap::Parser;
use fstools_formats::entryfilelist::EntryfilelistContainer;
use fstools_dvdbnd::{FileKeyProvider, DvdBnd};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    erpath: PathBuf,

    #[arg(long)]
    dictionary: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let er_path = args.erpath;

    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];

    let vfs = DvdBnd::create(archives.clone(), &keys).expect("unable to create vfs");

    let dictionary = std::fs::read_to_string(args.dictionary)?;
    let lines = dictionary
        .lines()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();

    lines.iter()
        .filter(|l| l.to_str().unwrap().ends_with("entryfilelist"))
        .for_each(|l| {
            println!("Parsing: {}", l.to_str().unwrap());

            let reader = vfs.open(l).expect("Could not open dvdbnd entry");
            let container = EntryfilelistContainer::parse(reader.data())
                .expect("Could not parse entryfilelist");

            let entryfilelist = container.decompress().unwrap();

            for string in entryfilelist.strings.iter() {
                println!(" - Referenced asset: {:?}", string);
            }
        });

    Ok(())
}
