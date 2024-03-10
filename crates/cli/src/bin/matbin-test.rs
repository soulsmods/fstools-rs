use std::{error::Error, io::Read, path::PathBuf};

use clap::Parser;
use fstools_formats::{bnd4::BND4, dcx::DcxHeader, matbin::Matbin};
use fstools_vfs::{FileKeyProvider, Vfs};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    erpath: PathBuf,
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

    let vfs = Vfs::create(archives.clone(), &keys).expect("unable to create vfs");
    let matbinbnd = vfs.open("/material/allmaterial.matbinbnd.dcx").unwrap();

    let (_, mut decoder) = DcxHeader::read(matbinbnd)?;

    let mut decompressed = Vec::with_capacity(decoder.hint_size());
    decoder.read_to_end(&mut decompressed)?;

    let mut cursor = std::io::Cursor::new(decompressed);
    let bnd4 = BND4::from_reader(&mut cursor)?;

    for file in bnd4.files.iter() {
        println!(" + Walking file {}", file.path);

        let start = file.data_offset as usize;
        let end = start + file.compressed_size as usize;

        let bytes = &bnd4.data[start..end];
        let matbin = Matbin::parse(bytes).unwrap();

        println!(
            "   - Source path: {}",
            matbin.source_path().unwrap().to_string().unwrap()
        );
        println!(
            "   - Shader path: {}",
            matbin.shader_path().unwrap().to_string().unwrap()
        );

        let parameters = matbin
            .parameters()
            .collect::<Result<Vec<_>, _>>()
            .expect("Could not collect samplers");
        for parameter in parameters.iter() {
            println!(
                "   - Parameter: {} = {:?}",
                parameter.name.to_string().unwrap(),
                parameter.value,
            );
        }

        let samplers = matbin
            .samplers()
            .collect::<Result<Vec<_>, _>>()
            .expect("Could not collect samplers");
        for sampler in samplers.iter() {
            println!(
                "   - Sampler: {} = {}",
                sampler.name.to_string().unwrap(),
                sampler.path.to_string().unwrap(),
            );
        }
    }

    Ok(())
}
