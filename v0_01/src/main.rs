//! The `ssg` command-line entry point.
//!
//! `main` is deliberately thin: it collects the arguments, asks the library
//! to parse them, runs the requested action, and maps the outcome to an exit
//! code. All of the real logic lives in the `ssg` library crate.

use std::env;
use std::process::ExitCode;

use ssg::config::{self, Action};
use ssg::error::Result;
use ssg::site;

fn main() -> ExitCode {
    // `env::args()` yields the program name first, so we skip it and keep
    // only what the user actually typed. `collect()` gathers the iterator
    // into a `Vec<String>`.
    let args: Vec<String> = env::args().skip(1).collect();

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // `eprintln!` writes to stderr, which keeps error messages out
            // of any output that a script might be capturing from stdout.
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Perform the requested action. Returning a `Result` keeps the error
/// handling in one place (see `main` above).
fn run(args: &[String]) -> Result<()> {
    match config::parse(args)? {
        Action::Help => {
            print!("{}", config::HELP);
            Ok(())
        }
        Action::Version => {
            // `env!` bakes the Cargo.toml version into the binary at
            // compile time, so there is no runtime lookup needed.
            println!("ssg {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Action::Build(config) => {
            let stats = site::build(&config)?;
            println!(
                "built {} page(s) and copied {} asset(s) into {}",
                stats.pages,
                stats.assets,
                config.output_dir.display()
            );
            Ok(())
        }
    }
}
