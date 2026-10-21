# Changelog

This file records what each version of `ssg` adds and, just as importantly,
**why**, with an eye toward a reader who is learning Rust. It is the project's
history in code.

---

## v0_01 — a working core

**Status:** alpha · first version.

`v0_01` establishes the whole shape of the project: a library of small,
single-purpose modules plus a thin binary, one external dependency, an example
site, and thorough documentation.

### What's new

- **A working static site generator.** Given a content directory, a template,
  and an output directory, it converts Markdown to HTML and produces a
  deployable folder tree.
- **Markdown → HTML** via the [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark)
  crate (CommonMark-compliant, streaming parser).
- **Template rendering** with two placeholders, `{{title}}` and `{{content}}`,
  hand-rolled with `str::replace` — no template-engine dependency.
- **Title extraction** from the first `# ` or `## ` heading, with a file-name
  fallback.
- **Directory mirroring** — nested folders in `content/` become nested folders
  in the output.
- **Static asset copying** — any non-`.md` file (CSS, images, `.nojekyll`,
  `CNAME`) is copied byte-for-byte.
- **Command-line interface** parsed by hand (no `clap`), with `build`, `help`,
  and `--version` plus `--content`/`--template`/`--output`/`--verbose`.
- **A single error enum** (`SsgError`) implementing `Display`, `Error`, and
  `From<io::Error>`.
- **Unit tests** for the pure logic (8 tests) and a doc-test.
- **Documentation**: [`USAGE.md`](USAGE.md) (how to use it),
  [`LEARNING_RUST.md`](LEARNING_RUST.md) (the code as a Rust lesson), and this
  changelog.

### Why these choices

Each decision in v0_01 is a teaching decision as much as an engineering one:

- **Library + binary split** (`lib.rs` vs `main.rs`) shows how a real Rust
  project separates reusable logic from the entry point, and makes the logic
  unit-testable.
- **One dependency.** Everything except Markdown parsing is standard library,
  so the reader sees the actual mechanics of CLI parsing, error handling,
  filesystem walking, and templating rather than a third-party API.
- **Hand-rolled errors** make the `Display`/`Error`/`From` traits visible; a
  later version might replace them with `thiserror`/`anyhow` once the reader
  has seen what those crates automate.
- **Fail fast** — the template is validated before any file is written, so a
  typo in a placeholder produces an error, not a silently wrong site.

### Known limitations (candidates for later versions)

- No base-URL handling for sites hosted in a subdirectory.
- No index/listing pages; navigation is written by hand.
- No front matter (per-page metadata).
- No clean build (stale output isn't removed).
- One shared template; no per-page or per-folder overrides.

### Roadmap

A rough plan for coming versions, in the order the limitations above suggest:

- **v0_02** — front matter + a configurable base URL (the two features that
  most affect real gh-pages deployments).
- **v0_03** — a `serve` command with a local dev server, or clean builds and
  sorted output.
- **later** — multiple templates, listing pages, and a switch to `clap` for
  the growing CLI.

---

## Formatting convention

Each future version should add a `## vX_YY` section above this one, following
the same four-part shape: **What's new**, **Why these choices**, **Known
limitations**, and **Roadmap**. This keeps the changelog useful both as a
reference and as a narrative of how the code evolved.
