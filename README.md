# gpr-rust

Gpr-Rust is a Rust binding for [gpr](https://github.com/AdaCore/gpr).
its goal is to provide an easy way to integrate Ada sources into the
Rust build process. It allows to select and build a GNAT project file
by parsing the project and providing all required information to call
gprbuild and link the resulting library.

## Dependencies

To build this project a working installation of [alire](https://alire.ada.dev/) is
required. The build script will automatically download the required dependencies.
The toolchain and dependencies used to build this crate are only available for
the crates build process. For any Ada projects that are being built with this
crate an appropirate toolchain needs to be provided.

Addionally the following native C libraries are required:

 - `libgmp-dev`
 - `libssl-dev`

## Usage

This library is intended to be used in build scripts. The basic process
consists of three steps.

- Loading the project:
```rust
let project = gpr::Project::load(Path::new("/path/to/project.gpr")).unwrap();
```
- Building the project:
```rust
let output = Command::new("gprbuild")
    .args(project.gprbuild_args().unwrap())
    .stderr(Stdio::inherit())
    .output()
    .unwrap();

if !output.status.success() {
    panic!();
}
```
- Providing cargo with the required linker flags:
```rust
println!(
    "cargo:rustc-link-search{}",
    project.library_dir().unwrap().to_str().unwrap()
);
println!(
    "cargo:rustc-link-lib={}={}",
    project.library_kind().unwrap(),
    project.library_name().unwrap()
);
```
- Additionally it can be helpful to tell cargo that changes in the Ada code also
should trigger a rerun:
```rust
for dir in project.source_dirs().unwrap() {
    println!(
        "cargo:rerun-if-changed={}", dir.as_str()
    );
}
```

Gpr-Rust doesn't need much configuration, most of the code needed is
boilerplate. If any additional options are required to build the Ada
project these can be added as arguments to the `gprbuild` command.

An example project is located in [`examples/ada_hello/build.rs`](examples/ada_hello/build.rs).
It can be tested by running:
```shell
cd examples/ada_hello
LD_LIBRARY_PATH=ada_hello/lib cargo run
```
