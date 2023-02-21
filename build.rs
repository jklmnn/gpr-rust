use git2::{ErrorCode, Repository, ResetType};
use std::{collections::HashMap, env, path::Path, process::Command};

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
                Err(e) => panic!("failed to open repository: {e}"),
            },
            _ => panic!("failed to clone repository: {e}"),
        },
    };
    let (object, reference) = match repo.revparse_ext(rev) {
        Ok(or) => or,
        Err(e) => panic!("failed to find rev: {e}"),
    };
    match match reference {
        Some(r) => repo.set_head(r.name().unwrap()),
        None => repo.set_head_detached(object.id()),
    } {
        Ok(_) => repo.reset(&object, ResetType::Hard, None).unwrap(),
        Err(e) => panic!("failed to check out rev: {e}"),
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
    let mut envs: HashMap<String, String> = env::vars()
        .filter(|e| !e.0.ends_with("ALIRE_PREFIX"))
        .collect();
    let alire_path = out_dir.join("gpr_rust_alire");
    if !alire_path.join("alire.toml").exists()
        && !Command::new("alr")
            .env_clear()
            .envs(&envs)
            .current_dir(out_dir.to_str().unwrap())
            .args(["--no-tty", "init", "--lib", "gpr_rust_alire"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success()
    {
        panic!("failed to create alire project");
    }
    if !Command::new("alr")
        .env_clear()
        .envs(&envs)
        .current_dir(alire_path.to_str().unwrap())
        .args(["--no-tty", "-n", "with", "libgpr2"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        //panic!("failed to add libgpr2");
    }
    if !Command::new("alr")
        .env_clear()
        .envs(&envs)
        .current_dir(alire_path.to_str().unwrap())
        .args(["--no-tty", "-n", "update"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to update alire project");
    }
    let output = Command::new("alr")
        .env_clear()
        .envs(&envs)
        .current_dir(alire_path.to_str().unwrap())
        .args(["--no-tty", "-n", "printenv", "--unix"])
        .output()
        .unwrap();
    if !output.status.success() {
        println!("failed to get alire environment");
    }
    let env_output = String::from_utf8(output.stdout).unwrap();
    for line in env_output.split('\n') {
        if !line.starts_with("export") {
            continue;
        }
        if let Some(exp) = line.split_once(' ') {
            if let Some(e) = exp.1.split_once('=') {
                envs.insert(
                    e.0.to_string(),
                    e.1.strip_prefix('"')
                        .unwrap()
                        .strip_suffix('"')
                        .unwrap()
                        .to_string(),
                );
            }
        }
    }
    if !Command::new("python3")
        .env_clear()
        .envs(&envs)
        .args(["-m", "virtualenv", venv_path.to_str().unwrap()])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success()
    {
        panic!("failed to create virtualenv");
    }
    envs.insert(
        String::from("VIRTUAL_ENV"),
        String::from(venv_path.to_str().unwrap()),
    );
    let env_path = venv_path.join("bin").to_str().unwrap().to_owned();
    envs.get_mut("PATH").unwrap().insert(0, ':');
    envs.get_mut("PATH").unwrap().insert_str(0, &env_path);
    if !Command::new("pip")
        .env_clear()
        .envs(&envs)
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
        .env_clear()
        .envs(&envs)
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
        gpr_project_path.push_str(gpp.as_str());
        envs.get_mut("GPR_PROJECT_PATH").unwrap().push(':');
        envs.get_mut("GPR_PROJECT_PATH")
            .unwrap()
            .push_str(&gpr_project_path);
    }
    if !Command::new("make")
        .env_clear()
        .envs(&envs)
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
        gpr_project_path.push_str(gpp.as_str());
        envs.get_mut("GPR_PROJECT_PATH").unwrap().push(':');
        envs.get_mut("GPR_PROJECT_PATH")
            .unwrap()
            .push_str(&gpr_project_path);
    }
    if !Command::new("gprbuild")
        .env_clear()
        .envs(&envs)
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
