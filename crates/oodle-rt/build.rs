use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=oodle_rt.hpp");
    println!("cargo:rerun-if-changed=oodle2.h");
    println!("cargo:rerun-if-changed=oodle2base.h");

    let bindings = bindgen::Builder::default()
        .header("oodle_rt.hpp")
        .rustified_enum("OodleLZ_FuzzSafe")
        .rustified_enum("OodleLZ_Verbosity")
        .rustified_enum("OodleLZ_Decode_ThreadPhase")
        .rustified_enum("OodleLZ_Compressor")
        .rustified_enum("OodleLZ_CheckCRC")
        .ignore_functions()
        .ignore_methods()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;

    let out_path = PathBuf::from(std::env::var("OUT_DIR")?);

    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
