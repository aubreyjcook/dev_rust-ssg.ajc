# ssg — User Guide (v0_01)

`ssg` is a minimal static site generator written in Rust. It turns a folder of
Markdown files into a complete, deployable website by:

1. converting each `.md` file to HTML,
2. wrapping that HTML in a shared template,
3. copying every non-Markdown file (stylesheets, images, `CNAME`, `.nojekyll`, …)
   through unchanged,
4. mirroring the folder structure into an output directory you can upload to
   GitHub Pages, Netlify, Cloudflare Pages, or any static host.

This guide covers installing and using the tool. For a tour of the code aimed
at people learning Rust, see [`LEARNING_RUST.md`](LEARNING_RUST.md). For what
changed in this version, see [`CHANGELOG.md`](CHANGELOG.md).

---

## Prerequisites

You need a Rust toolchain (the compiler and Cargo). Install it from
[rustup.rs](https://rustup.rs) if you don't have it. Check with:

```text
rustc --version
cargo --version
```

---

## Building the binary

From this directory (`v0_01/`):

```text
cargo build --release
```

The binary is written to `target/release/ssg` (or `target\release\ssg.exe` on
Windows). For day-to-day development you can also use `cargo run -- …`, which
builds and runs in one step.

---

## Command-line reference

```text
ssg <COMMAND> [OPTIONS]
```

| Command | Description |
| --- | --- |
| `build` | Build the site from Markdown source files. |
| `help`  | Print the full help message (same as `--help` or `-h`). |
| `--version` | Print the version number (`-V`). |

### `ssg build` options

| Flag | Description | Default |
| --- | --- | --- |
| `-c, --content <DIR>` | Directory of Markdown files and assets. | `content` |
| `-t, --template <FILE>` | HTML template containing `{{title}}` and `{{content}}`. | `template.html` |
| `-o, --output <DIR>` | Directory to write the generated site. | `dist` |
| `-v, --verbose` | Print each file as it is processed. | off |
| `-h, --help` | Print help for `build`. | — |

Every option has a default, so the simplest possible invocation is just:

```text
ssg build
```

### Examples

```text
# Use the conventional layout (content/, template.html, dist/)
ssg build

# Point at a different layout
ssg build --content ./site --template ./theme/layout.html --output ./public

# See every file as it is written
ssg build --verbose
```

---

## Directory layout

`ssg` is convention-based. The default layout is:

```text
your-site/
├── content/            # Markdown + assets (the input)
│   ├── index.md
│   ├── about.md
│   ├── posts/
│   │   └── hello-world.md
│   └── style.css
├── template.html       # the shared page layout (the input)
└── dist/               # the generated site (the output)
    ├── index.html
    ├── about.html
    ├── posts/
    │   └── hello-world.html
    └── style.css
```

Rules:

- A file ending in `.md` is **converted** to `.html` (so `index.md` →
  `index.html`).
- Any other file is **copied verbatim**, keeping its name and folder.
- Folders are mirrored one-for-one into the output directory.

---

## The template

The template is an ordinary HTML file with two special placeholders:

| Placeholder | Replaced with |
| --- | --- |
| `{{title}}` | The page's title (from its first Markdown heading, or the file name). |
| `{{content}}` | The HTML generated from the Markdown body. |

A minimal template:

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{{title}}</title>
</head>
<body>
  <main>{{content}}</main>
</body>
</html>
```

`ssg` refuses to build if the template is missing either placeholder, so a
typo like `{{content}}` → `{{conten}}` fails loudly instead of producing pages
with literal `{{conten}}` in them.

---

## Page titles

Each page's title (used for `{{title}}`) is chosen in this order:

1. The first `# ` (level-1) heading in the file.
2. The first `## ` (level-2) heading, if there is no level-1 heading.
3. The file name, prettified (`hello-world.md` → `Hello world`).

The heading itself is left in the body, so a page with a `# Title` heading
will show that heading in the content *and* use it in the `<title>`.

---

## Quick start with the bundled example

An example site ships in the [`example/`](../example) directory. To build it,
run from inside `example/`:

```text
cd example
cargo run --manifest-path ../Cargo.toml -- build
# or, if you already built the release binary:
#   ../target/release/ssg build
```

The generated site appears in `example/dist/`. Open `dist/index.html` in a
browser to see the result.

---

## Deploying to GitHub Pages

`ssg`'s output is plain static files, so hosting is just "upload the `dist`
folder". For GitHub Pages specifically:

1. Build the site.
2. Commit and push the contents of your output directory to a branch such as
   `gh-pages`, or configure your repository to publish from the `dist/` folder
   of `main`.

Two files are worth placing in your `content/` directory so they are copied
into the output automatically:

- **`.nojekyll`** — an empty file that tells GitHub Pages *not* to run Jekyll
  on your site. Without it, files or folders whose names start with an
  underscore can be ignored.
- **`CNAME`** — a text file containing your custom domain, if you use one.

Because they aren't `.md` files, `ssg` copies them through untouched.

### A note about link paths

The bundled example links its stylesheet with an absolute path
(`/style.css`). That works when the site is served from the *root* of a
domain. If your Pages site lives at `https://user.github.io/repo/`, absolute
paths point at the wrong place. Handling a configurable *base URL* is a known
v0_01 limitation and a candidate for v0_02 (see [`CHANGELOG.md`](CHANGELOG.md)).

---

## Known limitations in v0_01

- **No base-URL handling** — absolute paths in links/CSS aren't rewritten.
- **No index/listing pages** — pages aren't automatically linked together; you
  write the navigation by hand.
- **No front matter** — page titles come only from Markdown headings, not
  per-page metadata.
- **No clean builds** — the output directory is not emptied first, so deleted
  source files leave stale output behind.
- **Single template** — every page shares one layout; there are no per-page or
  per-folder template overrides.

These are deliberate: each is a natural next step for a future version (see
the roadmap in the changelog).
