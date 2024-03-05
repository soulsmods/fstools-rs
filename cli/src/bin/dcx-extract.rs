use std::io::{Read, Write};

use clap::Parser;
use format::{bnd4::BND4, dcx::Dcx};
use memmap2::MmapOptions;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    file: String,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let path = std::path::PathBuf::from(args.file);

    let dcx_file = std::fs::File::open(&path)?;
    let data = unsafe {
        MmapOptions::new()
            .populate()
            .map_copy_read_only(&dcx_file)?
    };

    let dcx = Dcx::parse(&data).unwrap();

    let mut decoder = dcx.create_decoder().expect("Could not create decoder");

    let mut decompressed = Vec::with_capacity(decoder.hint_size());
    decoder.read_to_end(&mut decompressed)?;

    let mut cursor = std::io::Cursor::new(decompressed);
    let bnd4 = BND4::from_reader(&mut cursor)?;

    let folder = format!(
        "{}/{}/",
        path.parent().unwrap().to_str().unwrap(),
        path.file_stem().unwrap().to_str().unwrap(),
    );

    for entry in bnd4.files.iter() {
        let trimmed_path = entry.path.replace("N:\\", "").replace('\\', "/");
        let output_path = std::path::PathBuf::from(folder.clone()).join(trimmed_path.as_str());

        let parent = output_path.parent().unwrap();
        std::fs::create_dir_all(parent)?;

        let bytes = entry.bytes(&mut cursor)?;

        let mut file = std::fs::File::create(&output_path)?;
        file.write_all(&bytes)?;
    }

    Ok(())
}
