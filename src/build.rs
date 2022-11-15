#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use std::process::Command;

fn main() {
    // For icon and version information

    // I can't use cfg!(target_family = "windows") because it's always Linux
    // when cross-compiling to windows
    // So I use a (undocumented?) env variable as suggested by
    // https://kazlauskas.me/entries/writing-proper-buildrs-scripts.html
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os.as_str() == "windows" {
        let out_dir = env::var("OUT_DIR").unwrap();
        Command::new("x86_64-w64-mingw32-windres")
            .args(&["src/program.rc"])
            .arg(&format!("{}/program.o", out_dir))
            .status()
            .unwrap();

        Command::new("x86_64-w64-mingw32-gcc-ar")
            .args(&["crus", "libprogram.a", "program.o"])
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();

        println!("cargo:rustc-link-search=native={}", out_dir);

        // In some rustc version 1.45.2 and 1.46.0.
        // Between 2021 and 2022, the linking of the resources file for windows broke. Apparently
        // since rustc version 1.61.0, the "+whole-archive" is necessary:
        // https://github.com/mxre/winres/issues/40
        println!("cargo:rustc-link-lib=static:+whole-archive=program");
    }
}
