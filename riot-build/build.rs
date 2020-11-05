use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let riotbase = env::var_os("RIOTBASE").unwrap().into_string().unwrap();
    let board = env::var_os("BOARD").unwrap().into_string().unwrap();

    let out_path = Path::new(&out_dir);
    let riot_builddir = Path::new(&out_dir).join("libriot");
    let riot_bindir = riot_builddir.join("bin");

    // assume they're nested
    fs::create_dir_all(&riot_bindir).unwrap();

    let app_name = "libriot";

    // create makefile for RIOT build system
    let mut makefile_content = format!(
        "APPLICATION={app_name}\n\
        BOARD={board}\n\
        RIOTBASE={riotbase}\n\
        include {crate_dir}/Makefile.riotbuild-rs\n",
        app_name = &app_name,
        board = &board,
        riotbase = &riotbase,
        crate_dir = &crate_dir
    );

    // if the riot_rs_core feature was set, configure the riot build accordingly
    if let Some(_) = env::var_os("CARGO_FEATURE_RIOT_RS_CORE") {
        let riot_rs_core_makefile = env::var_os("DEP_RIOT_RS_CORE_MAKEFILE").unwrap();

        makefile_content += &format!("include {}\n", riot_rs_core_makefile.to_string_lossy());
    }

    // include base RIOT Makefile, must be last in `makefile_content`.
    makefile_content += &format!("include {}/Makefile.include\n", riotbase);

    // finalize and write Makefile
    let makefile_content = makefile_content;
    fs::write(riot_builddir.join("Makefile"), &makefile_content)
        .expect("Couldn't write RIOT makefile!");

    // call out to RIOT build system
    let build_output = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "make -C {} clean afile QUIET=0",
            &riot_builddir.to_string_lossy()
        ))
        .output()
        .expect("failed to compile RIOT");

    // debug-print output
    eprint!(
        "{}",
        String::from_utf8_lossy(build_output.stdout.as_slice())
    );
    eprint!(
        "{}",
        String::from_utf8_lossy(build_output.stderr.as_slice())
    );

    // fetch archive created by RIOT build system
    let archive = riot_bindir.join(&board).join(format!("{}.a", app_name));
    eprintln!("archive: {}", archive.to_string_lossy());
    fs::copy(archive, Path::new(&out_dir).join("libriot.a")).unwrap();

    // instruct cargo to link RIOT archive
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=riot");
    println!(
        "cargo:MAKEFILE={}",
        out_path.join("Makefile").to_string_lossy()
    );

    // change notifiers
    println!("cargo:rerun-if-env-changed=BOARD");
    println!("cargo:rerun-if-env-changed=RIOTBASE");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RIOT_RS_CORE");
    println!("cargo:rerun-if-changed=Makefile.riotbuild-rs");
}
