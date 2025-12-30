use fs_extra::dir::{copy, CopyOptions};
use std::path::Path;
use std::{env, fs};

fn main() {
    // This tells Cargo to rerun this script if something in /res/ changes.
    let out_dir = env::var("OUT_DIR").unwrap();
    let src = Path::new("res");
    let dest = Path::new(&out_dir).join("res");

    println!("cargo:warning=src is {:?} and dest is {:?}", src, dest);

    if dest.exists() {
        fs::remove_dir_all(&dest).unwrap();
    }

    copy(
        src,
        &dest,
        &CopyOptions::new().overwrite(true).copy_inside(true),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=res");
}
