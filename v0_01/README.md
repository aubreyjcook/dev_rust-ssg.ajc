# ssg — v0_01

A minimal static site generator written in Rust, intended for deploying on
GitHub Pages and similar static hosts.

This directory is **version `v0_01`** of the project — the first working
version. It is intentionally small: a library of five single-purpose modules
plus a thin binary, with one external dependency (for Markdown parsing).

## What's here

| Path | Purpose |
| --- | --- |
| [`Cargo.toml`](Cargo.toml) | Project manifest and dependencies. |
| [`src/main.rs`](src/main.rs) | Command-line entry point. |
| [`src/lib.rs`](src/lib.rs) | Library root and module map. |
| [`src/error.rs`](src/error.rs) | The `SsgError` enum and `Result` alias. |
| [`src/config.rs`](src/config.rs) | Command-line parsing into a `Config`. |
| [`src/markdown.rs`](src/markdown.rs) | Markdown → HTML and title extraction. |
| [`src/template.rs`](src/template.rs) | `{{title}}`/`{{content}}` templating. |
| [`src/site.rs`](src/site.rs) | Walking the content tree and building. |
| [`example/`](example) | A runnable example site. |
| [`docs/USAGE.md`](docs/USAGE.md) | How to install and use `ssg`. |
| [`docs/LEARNING_RUST.md`](docs/LEARNING_RUST.md) | The code as a guided Rust lesson. |
| [`docs/CHANGELOG.md`](docs/CHANGELOG.md) | What this version adds and why. |

## Quick start

```text
cargo build --release
cd example
../target/release/ssg build
```

Then open `example/dist/index.html` in a browser. See
[`docs/USAGE.md`](docs/USAGE.md) for the full guide.
