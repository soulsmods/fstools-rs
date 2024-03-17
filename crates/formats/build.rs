use std::env;

fn main() {
    let project_dir = env::var("CARGO_MANIFEST_DIR").expect("cargo_manifest_dir");

    println!("cargo:rustc-link-search={}", project_dir); // the "-L" flag
    println!("cargo:rustc-link-lib=oo2corelinux64"); // the "-l" flag
}
