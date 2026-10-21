# Changelog

This file records what each version of `ssg` adds and, just as importantly,
**why**, with an eye toward a reader who is learning Rust. It is the project's
history in code.

---

## v0_02 — front matter and base URLs

**Status:** alpha · second version.

`v0_02` adds the two features v0_01's roadmap called for: per-page *front
matter* and a configurable *base URL* for subdirectory-hosted sites. Both make
the generator usable for real gh-pages deployments, and both introduce new Rust
ideas worth reading about in [`LEARNING_RUST.md`](LEARNING_RUST.md).

### What's new

- **Front matter** — a `---`-delimited block at the top of a Markdown file may
  set a `title` and/or a `template`. Titles override heading-based extraction;
  templates override the site-wide default for that one page.
- **Per-page templates** — pages can name their own layout; templates are
  cached in a `HashMap` so a shared layout is read from disk only once.
- **Base URL** — a new `--base-url` flag feeds an optional `{{base_url}}`
  template placeholder, so asset paths can be written once and work both at
  the domain root and in a subdirectory.
- **A new module** — `src/frontmatter.rs` parses the metadata block
  (hand-rolled, like the CLI parser).
- **More tests** — the suite grew from 8 to 15 unit tests.

### Why these choices

- **Hand-rolled front matter parser.** We support only scalar `key: value`
  lines and ignore unknown keys, which keeps the parser *total* (it never
  fails) and forward-compatible. A real project would use a YAML crate; that
  swap is noted in the roadmap below.
- **A `HashMap` template cache.** Re-reading a layout from disk for every page
  is wasteful, so templates are cached by path. This is also a natural place to
  meet Rust's `HashMap` collection and the borrow checker's rules around
  returning a reference out of a `&mut self` method.
- **`{{base_url}}` affects only the template, not Markdown content.** Rewriting
  links inside generated HTML is fragile; telling authors to use relative links
  in content is simple and correct. This scoping is documented explicitly in
  [`USAGE.md`](USAGE.md).

### Known limitations (candidates for later versions)

- Content links aren't rewritten (the base URL only touches the template).
- No index/listing pages.
- No per-folder template defaults.
- No clean builds (stale output isn't removed).
- Front matter is a tiny subset of YAML.

### Roadmap (for v0_03)

- **A `serve` command** — run a local development server that rebuilds (or at
  least serves) the site, so you don't have to open generated files by hand.
- **Clean builds and sorted output** — empty the output directory before
  building, and emit files in a deterministic (sorted) order.
- **Listing pages** — optionally generate an index that links pages together
  automatically.

Each of these builds on the tree-walking core in `src/site.rs` and is a small,
independently testable step.

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
