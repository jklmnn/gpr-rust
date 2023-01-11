use git2::{ErrorCode, Repository, ResetType};
use std::{env, path::Path, process::Command};

const GPR2_GIT: &str = "https://github.com/AdaCore/gpr.git";
const GPR2_REV: &str = "814f4654598dbc98db16dc47fb0e9f5cdeea4182";
const LANGKIT_GIT: &str = "https://github.com/AdaCore/langkit.git";
const LANGKIT_REV: &str = "a638facb03edb4baefdf8f1819db4ca56f191a5b";
const GPRCONFIG_KB_GIT: &str = "https://github.com/AdaCore/gprconfig_kb.git";
const GPRCONFIG_KB_REV: &str = "923c46ba4f0831d21dee4cd3d1179055e121de6c";

fn checkout(url: &str, rev: &str, path: &Path) {
    let path = path.to_str().unwrap();
    let repo = match Repository::clone(url, path) {
        Ok(repo) => repo,
        Err(e) => match e.code() {
            ErrorCode::Exists => match Repository::open(path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to open repository: {}", e),
            },
            _ => panic!("failed to clone repository: {}", e),
        },
    };
    let (object, reference) = match repo.revparse_ext(rev) {
        Ok(or) => or,
        Err(e) => panic!("failed to find rev: {}", e),
    };
    match match reference {
        Some(r) => repo.set_head(r.name().unwrap()),
        None => repo.set_head_detached(object.id()),
    } {
        Ok(_) => repo.reset(&object, ResetType::Hard, None).unwrap(),
        Err(e) => panic!("failed to check out rev: {}", e),
    };
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rerun-if-env-changed=PATH");
    let out_dir = Path::new(&out_dir);
    let contrib = out_dir.join("contrib");
    let gpr_path = contrib.join("gpr");
    println!(
        "cargo:rerun-if-changed={}",
        contrib.as_path().to_str().unwrap()
    );
    let langkit_path = contrib.join("langkit");
    let gprconfig_kb_path = contrib.join("gprconfig_kb");
    let venv_path = contrib.join("venv");
    checkout(GPR2_GIT, GPR2_REV, gpr_path.as_path());
    checkout(LANGKIT_GIT, LANGKIT_REV, langkit_path.as_path());
    checkout(
        GPRCONFIG_KB_GIT,
        GPRCONFIG_KB_REV,
        gprconfig_kb_path.as_path(),
    );
    if !Command::new("python3")
        .args(["-m", "virtualenv", venv_path.to_str().unwrap()])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to create virtualenv");
    }
    let mut env_path = venv_path.join("bin").to_str().unwrap().to_owned();
    env_path.push(':');
    env_path.push_str(env::var("PATH").unwrap().as_str());
    if !Command::new("pip")
        .env("VIRTUAL_ENV", &venv_path)
        .env("PATH", &env_path)
        .args(["install", "-e", langkit_path.to_str().unwrap()])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to install langkit");
    }
    if !Command::new("make")
        .env("VIRTUAL_ENV", &venv_path)
        .env("PATH", &env_path)
        .args(["-C", gpr_path.join("langkit").to_str().unwrap()])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to generate parser sources");
    }
    let mut gprconfig_db_path = String::from("GPR2KBDIR=");
    gprconfig_db_path.push_str(gprconfig_kb_path.join("db").as_path().to_str().unwrap());
    let mut gpr_project_path = langkit_path.join("support").to_str().unwrap().to_owned();
    gpr_project_path.push(':');
    if let Ok(gpp) = env::var("GPR_PROJECT_PATH") {
        gpr_project_path.push_str(gpp.as_str())
    }
    if !Command::new("make")
        .env("VIRTUAL_ENV", &venv_path)
        .env("PATH", &env_path)
        .env("GPR_PROJECT_PATH", gpr_project_path.as_str())
        .args([
            "-C",
            gpr_path.to_str().unwrap(),
            gprconfig_db_path.as_str(),
            "build-lib-static-pic",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to build libgpr2");
    }
    let gpr2c_path = gpr_path.join("bindings").join("c");
    let mut gpr_project_path = gpr_path.as_path().to_str().unwrap().to_owned();
    gpr_project_path.push(':');
    if let Ok(gpp) = env::var("GPR_PROJECT_PATH") {
        gpr_project_path.push_str(gpp.as_str())
    }
    if !Command::new("gprbuild")
        .env("GPR_PROJECT_PATH", gpr_project_path.as_str())
        .args([
            "-j0",
            "-p",
            "-P",
            gpr2c_path
                .join("gpr2_c_binding.gpr")
                .as_path()
                .to_str()
                .unwrap(),
            "-XGPR2_BUILD=release",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to build libgpr2c");
    }
    println!(
        "cargo:rustc-link-search={}",
        gpr2c_path
            .join("build")
            .join("release")
            .join("lib")
            .as_path()
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-link-lib=dylib=gpr2c");
}
