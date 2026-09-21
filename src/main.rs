use std::process::ExitCode;

use jgrep::{cli, run_cli};

fn main() -> ExitCode {
    match run_cli(cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("jgrep: {error:#}");
            ExitCode::from(2)
        }
    }
}
