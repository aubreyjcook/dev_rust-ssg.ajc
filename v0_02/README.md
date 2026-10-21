# ssg — v0_02

A minimal static site generator written in Rust, intended for deploying on
GitHub Pages and similar static hosts.

This directory is **version `v0_02`** of the project. It builds on `v0_01` by
adding per-page **front matter** and a configurable **base URL** for sites
hosted in a subdirectory. The code is a library of six single-purpose modules
plus a thin binary, with one external dependency (for Markdown parsing).

## What's here

| Path | Purpose |
| --- | --- |
| [`Cargo.toml`](Cargo.toml) | Project manifest and dependencies. |
| [`src/main.rs`](src/main.rs) | Command-line entry point. |
| [`src/lib.rs`](src/lib.rs) | Library root and module map. |
| [`src/error.rs`](src/error.rs) | The `SsgError` enum and `Result` alias. |
| [`src/config.rs`](src/config.rs) | Command-line parsing into a `Config`. |
| [`src/frontmatter.rs`](src/frontmatter.rs) | Per-page YAML-style metadata. |
| [`src/markdown.rs`](src/markdown.rs) | Markdown → HTML and title extraction. |
| [`src/template.rs`](src/template.rs) | `{{title}}`/`{{content}}`/`{{base_url}}` templating. |
| [`src/site.rs`](src/site.rs) | Walking the content tree, templates, and building. |
| [`example/`](example) | A runnable example site (incl. a second `post.html` layout). |
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

## What changed since v0_01

- **Front matter** — pages can set their own `title` and `template`.
- **Base URL** — `--base-url` + the `{{base_url}}` placeholder for
  subdirectory hosting.

See [`docs/CHANGELOG.md`](docs/CHANGELOG.md) for the details and the v0_03
roadmap.
