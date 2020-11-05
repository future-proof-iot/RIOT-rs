use cbindgen;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let out_path = Path::new(&out_dir);
    let gen_include_path = out_path.join("include");
    let include_path = Path::new(&crate_dir).join("include");

    // generate C header for C bindings
    let gen_header = gen_include_path.join("riot-rs-core.h");

    std::fs::create_dir_all(&gen_include_path).unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&*gen_header);

    // generate RIOT makefile snippet
    let makefile_content = format!(
        "CFLAGS += -DUSE_RUST_CORE=1\n\
         export USE_RUST_CORE = 1\n\
         DISABLE_MODULE += core\n\
         USEMODULE += core_idle_thread\n\
         INCLUDES += -I{}\n\
         INCLUDES += -I{}\n",
        gen_include_path.to_string_lossy(),
        include_path.to_string_lossy()
    );

    let makefile_name = "Makefile.riot-rs-core";
    fs::write(out_path.join(&makefile_name), &makefile_content)
        .expect("Couldn't write riot-rs-core makefile!");

    // let dependent crates know the location of our makefile snippet
    // This requires `links = "riot-core-rs"` in Cargo.toml of this package.
    println!(
        "cargo:MAKEFILE={}",
        out_path.join(&makefile_name).to_string_lossy()
    );
}
