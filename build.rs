use git2::{ErrorCode, Repository, ResetType};
use std::{collections::HashMap, env, ffi::OsStr, path::Path, process::Command};

const GPR2_GIT: &str = "https://github.com/jklmnn/gpr.git";
const GPR2_REV: &str = "4e88e9734194fc1ad58f19a45c95fa4f17dd475f";
const GPRCONFIG_KB_GIT: &str = "https://github.com/AdaCore/gprconfig_kb.git";
const GPRCONFIG_KB_REV: &str = "5a8f26e16ad42f84b4037a7c382b55e5491fbd2c";

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

fn call<'a, IE, IA, K, V>(
    cmd: &str,
    envs: IE,
    cwd: Option<&Path>,
    args: IA,
    panic_on_fail: bool,
) -> String
where
    IE: IntoIterator<Item = (K, V)> + Clone,
    IA: IntoIterator<Item = &'a str> + Clone,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut output = Command::new(cmd);
    output.env_clear();
    output.envs(envs.clone());
    if let Some(d) = cwd {
        output.current_dir(d.to_str().unwrap());
    }
    let output = output.args(args.clone()).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    for (k, v) in envs.into_iter() {
        println!(
            "{}={}",
            k.as_ref().to_str().unwrap(),
            v.as_ref().to_str().unwrap()
        );
    }
    println!("{}", &cmd);
    for a in args {
        println!("{}", a);
    }
    println!("{}", &stdout);
    println!("{}", &stderr);
    if !output.status.success() && panic_on_fail {
        panic!("failed to run command: {}\n{}", cmd, &stderr,);
    }
    stdout
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
    let gprconfig_kb_path = contrib.join("gprconfig_kb");
    checkout(GPR2_GIT, GPR2_REV, gpr_path.as_path());
    checkout(
        GPRCONFIG_KB_GIT,
        GPRCONFIG_KB_REV,
        gprconfig_kb_path.as_path(),
    );
    let mut envs: HashMap<String, String> = env::vars()
        .filter(|e| !e.0.ends_with("ALIRE_PREFIX"))
        .collect();
    let _ = call("alr", &envs, None, ["index", "--update-all"], true);
    let alire_path = out_dir.join("gpr_rust_alire");
    if !alire_path.join("alire.toml").exists() {
        let _ = call(
            "alr",
            &envs,
            Some(out_dir),
            ["--no-tty", "-n", "init", "--lib", "gpr_rust_alire"],
            true,
        );
    }
    let _ = call(
        "alr",
        &envs,
        Some(&alire_path),
        [
            "--no-tty",
            "-n",
            "with",
            "gnatcoll=25.0.0",
            "gnatcoll_iconv=25.0.0",
            "gnatcoll_gmp=25.0.0",
            "xmlada=25.0.0",
            "libgpr2=25.0.0",
        ],
        false,
    );
    let _ = call(
        "alr",
        &envs,
        Some(&alire_path),
        ["--no-tty", "-n", "update"],
        true,
    );
    let _ = call(
        "alr",
        &envs,
        Some(&alire_path),
        ["--no-tty", "-n", "build", "--", "-cargs", "-fPIC"],
        true,
    );
    let env_output = call(
        "alr",
        &envs,
        Some(&alire_path),
        ["--no-tty", "-n", "printenv", "--unix"],
        true,
    );
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
    let mut gprconfig_db_path = String::from("GPR2KBDIR=");
    gprconfig_db_path.push_str(gprconfig_kb_path.join("db").as_path().to_str().unwrap());
    let mut gpr_project_path = String::new();
    if let Ok(gpp) = env::var("GPR_PROJECT_PATH") {
        gpr_project_path.push_str(gpp.as_str());
        envs.get_mut("GPR_PROJECT_PATH").unwrap().push(':');
    }
    gpr_project_path.push(':');
    gpr_project_path.push_str(gpr_path.to_str().unwrap());
    envs.get_mut("GPR_PROJECT_PATH").unwrap().push(':');
    envs.get_mut("GPR_PROJECT_PATH")
        .unwrap()
        .push_str(&gpr_project_path);
    let _ = call(
        "make",
        &envs,
        None,
        [
            "-C",
            gpr_path.to_str().unwrap(),
            gprconfig_db_path.as_str(),
            "build-lib-static-pic",
        ],
        true,
    );
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
    let _ = call(
        "gprbuild",
        &envs,
        Some(&gpr_path),
        [
            "-j0",
            "-p",
            "-P",
            gpr2c_path
                .join("gpr2_c_binding.gpr")
                .as_path()
                .to_str()
                .unwrap(),
            "-XGPR2_BUILD=release",
            "-cargs",
            "-fPIC",
        ],
        true,
    );
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
