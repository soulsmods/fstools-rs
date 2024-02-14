use std::fs;
use std::env;

use flver::FLVER;

fn main() {
    // let args: Vec<String> = env::args().collect();
    //
    // if args.len() != 2 {
    //     panic!("Incorrect arguments to binary");
    // }
    //
    // let mut file = fs::File::open(&args[1])
    //     .expect("Could not open input FLVER file");

    let mut file = fs::File::open("./samples/c3251.flver")
        .expect("Could not open input FLVER file");


    let flver = FLVER::from_reader(&mut file).expect("Could not parse FLVER");
    println!("Output: {:#x?}", flver);
}
