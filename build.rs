use structopt::clap::Shell;

include!("src/cli.rs");

fn main() {
    let outdir = std::env::var("OUT_DIR").unwrap();
    let mut app = Command::clap();
    app.gen_completions("fedora-update-feedback", Shell::Bash, outdir);
}
