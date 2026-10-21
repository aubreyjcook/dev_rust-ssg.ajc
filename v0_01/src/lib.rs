//! `ssg` — a minimal static site generator.
//!
//! This crate is organised as a small library plus a thin binary. Splitting
//! the two means the interesting logic lives in reusable, testable modules,
//! while `main.rs` only wires the command line to the library.
//!
//! # Module map
//!
//! | Module | Responsibility |
//! | --- | --- |
//! | [`config`] | Parse command-line arguments into a [`config::Config`]. |
//! | [`error`]  | The crate-wide [`error::SsgError`] and [`error::Result`]. |
//! | [`markdown`] | Turn Markdown into HTML and extract page titles. |
//! | [`template`] | Load and render the `{{title}}` / `{{content}}` template. |
//! | [`site`]  | Walk the content tree and orchestrate the build. |
//!
//! For a guided tour of the code aimed at Rust learners, see
//! `docs/LEARNING_RUST.md` in this directory.

pub mod config;
pub mod error;
pub mod markdown;
pub mod site;
pub mod template;
