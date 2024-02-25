use std::{fs, io::Read, path::PathBuf};

use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use souls_vfs::{FileKeyProvider, Vfs};
use steamlocate::SteamDir;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    erpath: Option<PathBuf>,
    #[arg(long)]
    archive: Option<String>,
    #[arg(long)]
    dictionary: String,
}

const ER_APPID: u32 = 1245620;

fn locate_er_dir() -> PathBuf {
    let mut steamdir = SteamDir::locate().expect("steam installation not found");

    match steamdir.app(&ER_APPID) {
        Some(app) => app.path.join("Game"),
        None => panic!("couldn't find elden ring installation"),
    }
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let er_path = args.erpath.unwrap_or_else(locate_er_dir);

    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];

    let vfs = Vfs::create(archives.clone(), &keys).expect("unable to create vfs");

    let dictionary = std::fs::read_to_string(args.dictionary)?;
    let lines = dictionary
        .lines()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();

    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos:>7}/{len:7} {msg}")
        .expect("Could not create progress bar style");

    lines
        .par_iter()
        .progress_with_style(style)
        .filter(|l| !l.to_str().unwrap().starts_with('#') && !l.to_str().unwrap().is_empty())
        .for_each(|l| {
            let path = l.to_str().unwrap();

            match vfs.open(path) {
                Ok(mut entry) => {
                    let mut buffer = Vec::new();
                    entry
                        .read_to_end(&mut buffer)
                        .expect("Could not read from vfs to file buffer");

                    let output_path = std::path::PathBuf::from(format!("./extract/{}", path));
                    let directory = output_path.parent().unwrap();
                    fs::create_dir_all(directory).unwrap();
                    fs::write(output_path, buffer).unwrap();
                }
                Err(_) => {
                    // println!("Got error while extracting {} - {:?}", path, e);
                }
            }
        });

    Ok(())
}
