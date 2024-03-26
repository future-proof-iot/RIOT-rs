use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("buildinfo.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    let builder = env::var("CARGO_CFG_BUILDER").unwrap_or("unknown".into());

    writeln!(&mut file, "pub const BOARD: &str = \"{}\";", builder).unwrap();
}
