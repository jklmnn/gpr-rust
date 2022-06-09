use git2::{Repository, ErrorCode};
use std::{env, path::{Path, PathBuf}, process::Command};

const GPR2_GIT: &str = "https://github.com/AdaCore/gpr.git";
const GPR2_REV: &str = "5e78545ef5fc61dc0998ab8691982c967c349942";
const LANGKIT_GIT: &str = "https://github.com/AdaCore/langkit.git";
const LANGKIT_REV: &str = "5d11f106290b1c7917c96d97053a975e9c41b2bc";

fn checkout(url: &str, rev: &str, path: &Path)
{
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
        None => repo.set_head_detached(object.id())
        } {
            Ok(_) => (),
            Err(e) => panic!("failed to check out rev: {}", e),
        };
}

fn main()
{
    let mut gpr_path = PathBuf::new();
    gpr_path.push(env::var("OUT_DIR").unwrap());
    gpr_path.push("contrib");
    let mut langkit_path = gpr_path.clone();
    let mut venv_path = gpr_path.clone();
    gpr_path.push("gpr");
    langkit_path.push("langkit");
    venv_path.push("venv");
    checkout(GPR2_GIT, GPR2_REV, gpr_path.as_path());
    checkout(LANGKIT_GIT, LANGKIT_REV, langkit_path.as_path());
    let output = Command::new("python3").args(["-m", "virtualenv", venv_path.to_str().unwrap()])
        .spawn().unwrap()
        .wait_with_output().unwrap();
    if !output.status.success() {
        panic!("failed to create virtualenv");
    }
    let env_venv = venv_path.to_str().unwrap().to_owned();
    venv_path.push("bin");
    let mut env_path = venv_path.to_str().unwrap().to_owned();
    env_path.push_str(":");
    env_path.push_str(env::var("PATH").unwrap().as_str());
    let output = Command::new("pip")
        .env("VIRTUAL_ENV", env_venv)
        .env("PATH", env_path)
        .args(["install", "-e", langkit_path.to_str().unwrap()])
        .spawn().unwrap()
        .wait_with_output().unwrap();
    if !output.status.success() {
        panic!("failed to install langkit");
    }
}
