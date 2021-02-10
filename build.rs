use std::env;
use std::process::Command;
use structopt::clap::Shell;

include!("src/cli.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };
    let mut app = ProgramOptions::clap();

    app.gen_completions("girouette", Shell::Bash, &outdir);

    app.gen_completions("girouette", Shell::Zsh, &outdir);

    app.gen_completions("girouette", Shell::Fish, outdir);

    if let Some(v) = version_check::Version::read() {
        println!("cargo:rustc-env=BUILD_RUSTC={}", v)
    }

    if let Some(hash) = get_commit_hash().or_else(|| env::var("BUILD_ID").ok()) {
        println!("cargo:rustc-env=BUILD_ID={}", hash);
    }

    println!(
        "cargo:rustc-env=BUILD_INFO={}-{}-{}-{}",
        env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
        env::var("CARGO_CFG_TARGET_VENDOR").unwrap(),
        env::var("CARGO_CFG_TARGET_OS").unwrap(),
        env::var("CARGO_CFG_TARGET_ENV").unwrap(),
    );
}

fn get_commit_hash() -> Option<String> {
    Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|r| {
            if r.status.success() {
                String::from_utf8(r.stdout).ok()
            } else {
                None
            }
        })
}
