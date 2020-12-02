//use cbindgen;
use std::env;

fn main() {
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("riotcore.h");
}
