use std::io::Write;

use clap::Parser;
use format::{bnd4::BND4, dcx::DCX};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    file: String,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let path = std::path::PathBuf::from(args.file);

    let mut dcx_file = std::fs::File::open(&path)?;
    let dcx = DCX::from_reader(&mut dcx_file)?;

    let mut cursor = std::io::Cursor::new(dcx.decompressed);
    let bnd4 = BND4::from_reader(&mut cursor)?;

    let folder = format!(
        "{}/{}/",
        path.parent().unwrap().to_str().unwrap(),
        path.file_stem().unwrap().to_str().unwrap(),
    );

    for entry in bnd4.file_descriptors.iter() {
        let trimmed_path = entry.name.replace("N:\\", "").replace("\\", "/");

        let output_path = std::path::PathBuf::from(folder.clone())
            .join(trimmed_path.as_str());

        let parent = output_path.parent().unwrap();
        std::fs::create_dir_all(parent)?;

        let bytes = entry.bytes(&mut cursor)?;

        let mut file = std::fs::File::create(&output_path)?;
        file.write_all(&bytes)?;
    }

    Ok(())
}
