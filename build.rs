use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/cli.rs");

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let bin_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let mut command = Command::command();
    generate_to(Shell::Bash, &mut command, &bin_name, &out_dir).unwrap();
    generate_to(Shell::Fish, &mut command, &bin_name, &out_dir).unwrap();
    generate_to(Shell::Zsh, &mut command, &bin_name, &out_dir).unwrap();
}
