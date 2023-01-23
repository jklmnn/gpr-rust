use gpr::Project;
use std::{
    path::Path,
    process::{Command, Stdio},
};

fn main() {
    let ada_hello = Project::load(Path::new("ada_hello/ada_hello.gpr")).unwrap();
    let output = Command::new("gprbuild")
        .args(ada_hello.gprbuild_args().unwrap())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    if !output.status.success() {
        panic!();
    }

    println!(
        "cargo:rustc-link-search={}",
        ada_hello.library_dir().unwrap().to_str().unwrap()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ada_hello.source_dirs().unwrap()[0].as_str()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ada_hello.library_dir().unwrap().to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-lib={}={}",
        ada_hello.library_kind().unwrap(),
        ada_hello.library_name().unwrap()
    );
}
