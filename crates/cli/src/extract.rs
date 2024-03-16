use std::{error::Error, fs, io::Read, path::PathBuf};

use fstools_dvdbnd::DvdBnd;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;

pub fn extract(
    dvd_bnd: DvdBnd,
    _recursive: bool,
    filter: Option<String>,
    output_path: PathBuf,
    dictionary_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let dictionary = fs::read_to_string(dictionary_path)?;
    let lines = dictionary
        .lines()
        .filter(|line| {
            !line.starts_with('#')
                && !line.is_empty()
                && filter
                    .as_ref()
                    .map(|filter| line.contains(filter))
                    .unwrap_or(true)
        })
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();

    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos:>7}/{len:7} {msg}")
        .expect("Could not create progress bar style");

    lines.par_iter().progress_with_style(style).for_each(|l| {
        let path = l.to_str().unwrap();

        match dvd_bnd.open(path) {
            Ok(mut entry) => {
                let mut buffer = Vec::new();
                entry
                    .read_to_end(&mut buffer)
                    .expect("Could not read from dvdbnd to file buffer");

                let fs_path = output_path.join(path);
                if let Some(directory) = fs_path.parent() {
                    let _ = fs::create_dir_all(directory);
                }

                fs::write(fs_path, buffer).expect("failed to write data");
            }
            Err(_) => {
                // println!("Got error while extracting {} - {:?}", path, e);
            }
        }
    });

    Ok(())
}
