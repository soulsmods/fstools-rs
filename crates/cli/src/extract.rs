use std::{
    error::Error,
    fs,
    io::{Cursor, Read},
    path::PathBuf,
};

use fstools_dvdbnd::DvdBnd;
use fstools_formats::{bnd4::BND4, dcx::DcxHeader};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;

pub trait SyncError: Sync + Send + Error + Sized {}

pub fn extract(
    dvd_bnd: &DvdBnd,
    recursive: bool,
    filter: Option<String>,
    output_path: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let lines = fstools_elden_ring_support::dictionary()
        .filter(|line| {
            filter
                .as_ref()
                .map(|filter| line.to_string_lossy().contains(filter))
                .unwrap_or(true)
        })
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();

    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos:>7}/{len:7} {msg}")
        .expect("Could not create progress bar style");

    let result = lines
        .par_iter()
        .progress_with_style(style)
        .try_fold(
            || 0usize,
            |total, path| {
                match dvd_bnd.open(path.to_string_lossy().as_ref()) {
                    Ok(mut reader) => {
                        let is_archive = recursive && path.to_string_lossy().ends_with("bnd.dcx");
                        let path = path.strip_prefix("/").expect("no leading slash");
                        let parent_path = if is_archive {
                            // twice to strip "bnd.dcx"
                            output_path.join(path.with_extension("").with_extension(""))
                        } else {
                            output_path.to_path_buf()
                        };

                        let _ = fs::create_dir_all(&parent_path);

                        if is_archive {
                            let (_, mut dcx_reader) = DcxHeader::read(reader)?;
                            let mut buffer = Vec::new();
                            dcx_reader.read_to_end(&mut buffer)?;

                            let bnd4 = BND4::from_reader(Cursor::new(&buffer))?;

                            for file in bnd4.files {
                                let last_sep =
                                    file.path.rfind('\\').map(|index| index + 1).unwrap_or(0);

                                let output_path = parent_path.join(&file.path[last_sep..]);

                                let offset = file.data_offset as usize;
                                let size = file.compressed_size as usize;

                                fs::write(output_path, &buffer[offset..offset + size])?;
                            }

                            Ok::<_, Box<dyn Error + Send + Sync>>(total + bnd4.file_count as usize)
                        } else {
                            let mut buffer = Vec::new();
                            reader.read_to_end(&mut buffer)?;

                            fs::write(
                                parent_path.join(path.file_name().expect("no file name")),
                                buffer,
                            )?;

                            Ok::<_, Box<dyn Error + Send + Sync>>(total + 1)
                        }
                    }
                    Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
                }
            },
        )
        .try_reduce(|| 0, |a, b| Ok(a + b));

    match result {
        Ok(count) => {
            println!("Extracted {count} files");
            Ok(())
        }
        Err(e) => Err(e as Box<dyn Error>),
    }
}
