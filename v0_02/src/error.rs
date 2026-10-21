//! Error handling shared by the whole crate.
//!
//! # Why a custom error type?
//!
//! Rust represents failure with the [`Result`] type, which is generic over
//! two type parameters: the success value `T` and the error value `E`.
//! A function that can fail therefore *says so in its signature*, e.g.
//!
//! ```ignore
//! fn load(path: &Path) -> Result<Template, SsgError>;
//! ```
//!
//! Early Rust code often uses `Box<dyn Error>` as `E`, which means "any
//! error at all". That is quick to write but hides what can actually go
//! wrong. A dedicated enum like [`SsgError`] is more work up front, but it
//! documents every failure mode and lets callers `match` on it precisely.
//!
//! For a small program like this one the enum stays tiny. In a larger
//! project you would reach for a crate such as `thiserror` or `anyhow` to
//! remove the boilerplate; we hand-roll it here so the boilerplate is
//! visible and explained.

use std::fmt;

/// Every way a build can go wrong, in one place.
#[derive(Debug)]
pub enum SsgError {
    /// A command-line word we don't recognise (for example `ssg frobnicate`).
    UnknownCommand(String),
    /// A `--flag` we don't recognise on the `build` command.
    UnknownOption(String),
    /// A flag that needs a value (`--content`, `--template`, `--output`)
    /// was given without one.
    MissingValue(String),
    /// A filesystem operation failed (missing file, permission denied, ...).
    Io(std::io::Error),
    /// The template does not contain a required `{{...}}` placeholder.
    MissingPlaceholder(String),
    /// A path the user supplied does not exist.
    NotFound(String),
}

impl fmt::Display for SsgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsgError::UnknownCommand(cmd) => write!(f, "unknown command: {cmd}"),
            SsgError::UnknownOption(opt) => write!(f, "unknown option: {opt}"),
            SsgError::MissingValue(flag) => write!(f, "missing value for {flag}"),
            SsgError::Io(err) => write!(f, "i/o error: {err}"),
            SsgError::MissingPlaceholder(name) => {
                write!(f, "template is missing the {{{name}}} placeholder")
            }
            SsgError::NotFound(path) => write!(f, "not found: {path}"),
        }
    }
}

impl std::error::Error for SsgError {
    // `source` lets error *chains* work. If our error wraps another error,
    // we hand it back so tools (and `{:?}` formatting) can show the original
    // cause. Errors that are not a wrapper return `None`.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SsgError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// The `From` trait lets Rust *convert* one type into another automatically.
// Implementing `From<io::Error> for SsgError` means the `?` operator can
// convert a filesystem error into our error type with no extra code, which
// is why call sites can write `fs::read_to_string(path)?` directly.
impl From<std::io::Error> for SsgError {
    fn from(err: std::io::Error) -> Self {
        SsgError::Io(err)
    }
}

/// The crate-wide [`Result`] alias: a `Result` whose error is always
/// [`SsgError`]. This is just a convenience so call sites read
/// `Result<T>` instead of `Result<T, SsgError>` everywhere.
pub type Result<T> = std::result::Result<T, SsgError>;
