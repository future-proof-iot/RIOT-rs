use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fs::{copy, create_dir_all, write, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let riotbase = env::var("RIOTBASE").expect("Error getting RIOTBASE env variable");
    let board = env::var("BOARD").expect("Error getting BOARD env variable");

    let riot_app_mode = env::var("CARGO_FEATURE_RIOT_APP").is_ok();

    let (app_name, riot_bindir, riot_builddir, build_output) = if riot_app_mode {
        // building a RIOT application
        let app = env::var("APP").unwrap();
        eprintln!("riot-build: RIOT C application mode, APP=\"{}\"", &app);
        let riot_builddir = Path::new(&riotbase).join(app);
        let riot_bindir = riot_builddir.join("bin");

        let mut riot_extra_makefiles = vec![format!("{}/Makefile.riotbuild-rs", &crate_dir)];

        // if the riot_rs_core feature was set, configure the riot build accordingly
        if let Some(_) = env::var_os("CARGO_FEATURE_RIOT_RS_CORE") {
            let riot_rs_core_makefile = env::var_os("DEP_RIOT_RS_CORE_MAKEFILE").unwrap();
            riot_extra_makefiles.push(format!("{}", riot_rs_core_makefile.to_string_lossy()));
        }

        let mut riot_make_env = HashMap::<OsString, OsString>::new();
        riot_make_env.insert(
            "RIOT_MAKEFILES_GLOBAL_PRE".into(),
            riot_extra_makefiles.join(" ").into(),
        );

        fn get_riot_var(riot_builddir: &str, var: &str) -> String {
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "make --no-print-directory -C {} info-debug-variable-{}",
                    riot_builddir, var
                ))
                .output()
                .unwrap()
                .stdout;
            String::from_utf8_lossy(output.as_slice()).trim_end().into()
        }

        let app_name = get_riot_var(&*riot_builddir.to_string_lossy(), "APPLICATION");
        // call out to RIOT build system
        let build_output = Command::new("sh")
            .arg("-c")
            .envs(&riot_make_env)
            .arg(format!(
                "make -C {} clean afile QUIET=0",
                &riot_builddir.to_string_lossy()
            ))
            .output()
            .expect("failed to compile RIOT");

        (app_name, riot_bindir, riot_builddir, build_output)
    } else {
        // building RIOT as library
        eprintln!("riot-build: Rust application mode");
        let riot_builddir = Path::new(&out_dir).join("libriot");
        let riot_bindir = riot_builddir.join("bin");

        create_dir_all(&riot_builddir).unwrap();
        create_dir_all(&riot_bindir).unwrap();

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

        if let Some(usemodule) = env::var_os("USEMODULE") {
            makefile_content += &format!("USEMODULE += {}\n", &usemodule.into_string().unwrap());
        }
        // if the riot_rs_core feature was set, configure the riot build accordingly
        if let Some(_) = env::var_os("CARGO_FEATURE_RIOT_RS_CORE") {
            let riot_rs_core_makefile = env::var_os("DEP_RIOT_RS_CORE_MAKEFILE").unwrap();

            makefile_content += &format!("include {}\n", riot_rs_core_makefile.to_string_lossy());
        }
        // include base RIOT Makefile, must be last in `makefile_content`.
        makefile_content += &format!("include {}/Makefile.include\n", riotbase);

        // finalize and write Makefile
        let makefile_content = makefile_content;
        write(riot_builddir.join("Makefile"), &makefile_content)
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

        (
            String::from(app_name),
            riot_bindir,
            riot_builddir,
            build_output,
        )
    };

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
    copy(archive, Path::new(&out_dir).join("libriot.a")).unwrap();

    // instruct cargo to link RIOT archive
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=riot");

    // instruct cargo to link in newlib
    println!(
        "cargo:rustc-link-arg=-L/usr/arm-none-eabi/lib/{}",
        env::var("NEWLIB_ARCH").expect("missing NEWLIB_ARCH")
    );
    println!("cargo:rustc-link-arg=-lc_nano");

    // with `links = "riot-build", this results in
    // DEP_RIOT_BUILD_DIR=foo being passed to dependees
    println!("cargo:DIR={}", riot_builddir.to_string_lossy());

    // xfa support
    File::create(PathBuf::from(out_dir).join("xfa.ld"))
        .unwrap()
        .write_all(include_bytes!("xfa.ld"))
        .unwrap();
    println!("cargo:rerun-if-changed=xfa.ld");
    println!("cargo:rustc-link-arg=-Txfa.ld");

    // change notifiers
    println!("cargo:rerun-if-env-changed=APP");
    println!("cargo:rerun-if-env-changed=BOARD");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RIOT_APP");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_RIOT_RS_CORE");
    println!("cargo:rerun-if-env-changed=RIOTBASE");
    println!("cargo:rerun-if-env-changed=USEMODULE");
    println!("cargo:rerun-if-changed=Makefile.riotbuild-rs");
}
