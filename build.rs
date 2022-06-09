use git2::{Repository, ErrorCode};
use std::{env, path::{Path, PathBuf}};

const GPR2_GIT: &str = "https://github.com/AdaCore/gpr.git";
const GPR2_REV: &str = "5e78545ef5fc61dc0998ab8691982c967c349942";
const LANGKIT_GIT: &str = "https://github.com/AdaCore/langkit.git";
const LANGKIT_REV: &str = "5d11f106290b1c7917c96d97053a975e9c41b2bc";

fn checkout(url: &str, rev: &str, path: &Path)
{
    let path = match path.to_str() {
        Some(p) => p,
        None => panic!("failed to decode path"),
    };
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
    gpr_path.push("gpr");
    langkit_path.push("langkit");
    let _gpr2_repo = checkout(GPR2_GIT, GPR2_REV, gpr_path.as_path());
    let _langkit_repo = checkout(LANGKIT_GIT, LANGKIT_REV, langkit_path.as_path());
}
