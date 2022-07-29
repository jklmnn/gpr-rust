use gpr::Project;
use std::{path::Path, process::Command};

fn main() {
    let ada_hello = Project::load(Path::new("ada_hello/ada_hello.gpr")).unwrap();
    Command::new("gprbuild")
        .args(ada_hello.gprbuild_args().unwrap())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    println!(
        "cargo:rustc-link-search={}",
        ada_hello.library_dir().unwrap().to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-lib={}={}",
        ada_hello.library_kind().unwrap(),
        ada_hello.library_name().unwrap()
    );
}
