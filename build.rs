use std::env;

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
}
