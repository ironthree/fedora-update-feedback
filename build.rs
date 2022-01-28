use structopt::clap::Shell;

include!("src/cli.rs");

fn main() {
    let outdir = concat!(env!("CARGO_MANIFEST_DIR"), "/etc/");
    let _ = std::fs::create_dir(outdir);

    let mut app = Command::clap();
    app.gen_completions("fedora-update-feedback", Shell::Bash, outdir);
}
