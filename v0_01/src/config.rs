//! Command-line argument parsing.
//!
//! `v0_01` deliberately avoids the popular `clap` crate so that the parsing
//! logic is small enough to read top-to-bottom. Hand-rolling it also shows
//! how the same `match` and `Option` machinery you already know can drive a
//! command-line interface.
//!
//! The parsing is deliberately split from the *meaning* of the arguments:
//! [`parse`] produces an [`Action`], and everything downstream only ever
//! looks at the plain [`Config`] struct. That separation means a future
//! version could swap this hand-written parser for `clap` without touching
//! the rest of the program.

use std::path::PathBuf;

use crate::error::{Result, SsgError};

/// The top-level thing the user asked the program to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Build the site with the given configuration.
    Build(Config),
    /// Print usage information and exit successfully.
    Help,
    /// Print the version number and exit successfully.
    Version,
}

/// Everything the builder needs to know to do its job.
///
/// Every field has a sensible default, so a bare `ssg build` works against
/// the conventional layout:
///
/// ```text
/// content/        <- Markdown + assets  (default content dir)
/// template.html   <- page layout        (default template)
/// dist/           <- generated site     (default output dir)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Directory that holds the Markdown source files and static assets.
    pub content_dir: PathBuf,
    /// Path to the single HTML template with `{{title}}` / `{{content}}`.
    pub template_path: PathBuf,
    /// Directory where the generated site is written.
    pub output_dir: PathBuf,
    /// Print each processed file as we go, for debugging.
    pub verbose: bool,
}

// `Default` is a standard trait that provides a "default value" for a type.
// Implementing it lets us write `Config::default()` and also unlock helpers
// like `Option::unwrap_or_default`.
impl Default for Config {
    fn default() -> Self {
        Config {
            content_dir: PathBuf::from("content"),
            template_path: PathBuf::from("template.html"),
            output_dir: PathBuf::from("dist"),
            verbose: false,
        }
    }
}

/// Parse the program's command-line arguments into an [`Action`].
///
/// `args` is expected to be the arguments *without* the program name, which
/// is why `main` passes `env::args().skip(1)`.
pub fn parse(args: &[String]) -> Result<Action> {
    // `args.first()` returns `Option<&String>` (None when the slice is
    // empty). `.map(String::as_str)` converts the `&String` to a `&str`.
    match args.first().map(String::as_str) {
        None | Some("help") | Some("--help") | Some("-h") => Ok(Action::Help),
        Some("--version") | Some("-V") => Ok(Action::Version),
        Some("build") => parse_build(&args[1..]),
        Some(other) => Err(SsgError::UnknownCommand(other.to_string())),
    }
}

/// Parse the flag list that follows the `build` subcommand.
fn parse_build(args: &[String]) -> Result<Action> {
    let mut config = Config::default();

    // Walk the flags with an index because some flags consume the *next*
    // argument as their value (e.g. `--content some_dir`).
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--content" | "-c" => {
                i += 1; // move past the flag so `require_value` reads its value
                config.content_dir = PathBuf::from(require_value(args, i, "--content")?);
            }
            "--template" | "-t" => {
                i += 1;
                config.template_path = PathBuf::from(require_value(args, i, "--template")?);
            }
            "--output" | "-o" => {
                i += 1;
                config.output_dir = PathBuf::from(require_value(args, i, "--output")?);
            }
            "--verbose" | "-v" => config.verbose = true,
            "--help" | "-h" => return Ok(Action::Help),
            other => return Err(SsgError::UnknownOption(other.to_string())),
        }
        i += 1;
    }

    Ok(Action::Build(config))
}

/// Fetch the value that must follow a flag such as `--content <dir>`.
///
/// Returns an error when the flag is the last argument, so the user gets a
/// clear message instead of a crash.
fn require_value(args: &[String], index: usize, flag: &str) -> Result<String> {
    args.get(index)
        .cloned()
        .ok_or_else(|| SsgError::MissingValue(flag.to_string()))
}

/// The help text shown by `ssg help` (and on `ssg` with no arguments).
pub const HELP: &str = "\
ssg - a minimal static site generator

USAGE:
    ssg <COMMAND> [OPTIONS]

COMMANDS:
    build       Build the static site from Markdown source files.
    help        Print this help message.

BUILD OPTIONS:
    -c, --content <DIR>     Directory of Markdown files and assets.  [default: content]
    -t, --template <FILE>   HTML template with {{title}} and {{content}}. [default: template.html]
    -o, --output <DIR>      Directory to write the generated site.   [default: dist]
    -v, --verbose           Print each file as it is processed.
    -h, --help              Print this help message.

OTHER:
    -V, --version           Print the version number.

EXAMPLE:
    ssg build
    ssg build --content ./site --template ./theme/layout.html --output ./public
";
