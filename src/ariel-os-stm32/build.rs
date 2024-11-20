use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    // handle CONFIG_SWI
    {
        let dest_path = Path::new(&out_dir).join("swi.rs");
        if let Ok(var) = env::var("CONFIG_SWI") {
            fs::write(
                &dest_path,
                format!("ariel_os_embassy_common::executor_swi!({});\n", var).as_bytes(),
            )
            .expect("write failed");
        } else {
            fs::write(
                &dest_path,
                b"compile_error!(\"swi.rs included but CONFIG_SWI not set!\");\n",
            )
            .expect("write failed");
        }

        println!("cargo::rerun-if-env-changed=CONFIG_SWI");
    }
}
