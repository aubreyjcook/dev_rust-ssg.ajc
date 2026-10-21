# ssg — User Guide (v0_02)

`ssg` is a minimal static site generator written in Rust. It turns a folder of
Markdown files into a complete, deployable website by:

1. converting each `.md` file to HTML,
2. reading optional *front matter* (per-page `title` and `template`),
3. wrapping that HTML in a template — shared by default, overridable per page,
4. copying every non-Markdown file (stylesheets, images, `CNAME`, `.nojekyll`, …)
   through unchanged,
5. mirroring the folder structure into an output directory you can upload to
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

From this directory (`v0_02/`):

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
| `-t, --template <FILE>` | Default HTML template with `{{title}}`, `{{content}}`, and optional `{{base_url}}`. | `template.html` |
| `-o, --output <DIR>` | Directory to write the generated site. | `dist` |
| `-b, --base-url <URL>` | Base URL for the `{{base_url}}` placeholder. | (empty) |
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

# Build for a site hosted in a subdirectory
ssg build --base-url /my-repo

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

## Front matter

A Markdown file may begin with a small metadata block delimited by lines that
contain exactly `---`:

```text
---
title: My page
template: post.html
---
# My page
```

Two fields are recognised:

| Field | Meaning |
| --- | --- |
| `title` | Overrides the page title (beats any Markdown heading). |
| `template` | A template file to use for this page instead of the default. |

Rules and notes:

- The block must be at the **very top** of the file, and must have a closing
  `---` line. A lone leading `---` (for example a horizontal rule) is left as
  content, not treated as front matter.
- Each line is a `key: value` pair. Values may be quoted (`"..."` or `'...'`);
  surrounding matching quotes are stripped.
- Unknown keys are ignored, so you can leave extra metadata for a future
  version to understand.
- A `template` value is resolved **relative to the default template's folder**.
  With `--template ./theme/layout.html`, a page that says `template: post.html`
  uses `./theme/post.html`. Absolute paths are used as-is.

---

## The template

The template is an ordinary HTML file with these placeholders:

| Placeholder | Replaced with | Required? |
| --- | --- | --- |
| `{{title}}` | The page's title (front matter, heading, or file name). | yes |
| `{{content}}` | The HTML generated from the Markdown body. | yes |
| `{{base_url}}` | The configured base URL (often empty). | no |

A minimal template:

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{{title}}</title>
  <link rel="stylesheet" href="{{base_url}}/style.css">
</head>
<body>
  <main>{{content}}</main>
</body>
</html>
```

`ssg` refuses to build if a template is missing either required placeholder, so
a typo like `{{content}}` → `{{conten}}` fails loudly instead of producing pages
with literal `{{conten}}` in them. A placeholder may appear more than once; the
bundled `post.html` layout uses `{{title}}` in both the `<title>` and a visible
`<h1>`.

---

## Page titles

Each page's title (used for `{{title}}`) is chosen in this order:

1. The `title:` field in the page's front matter, if present.
2. The first `# ` (level-1) heading in the file.
3. The first `## ` (level-2) heading, if there is no level-1 heading.
4. The file name, prettified (`hello-world.md` → `Hello world`).

A heading is left in the body, so a page with a `# Title` heading will show
that heading in the content *and* use it in the `<title>` (unless front matter
overrides the title).

---

## Base URL (subdirectory hosting)

When a site is hosted in a subdirectory — for example GitHub Pages for a
project repo, at `https://user.github.io/my-repo/` — root-relative paths like
`/style.css` point at the wrong place.

`ssg` handles this with the `--base-url` flag and the `{{base_url}}` template
placeholder:

```text
ssg build --base-url /my-repo
```

In the template, write asset and navigation paths through the placeholder:

```html
<link rel="stylesheet" href="{{base_url}}/style.css">
<a href="{{base_url}}/">Home</a>
```

With no `--base-url`, `{{base_url}}` is replaced with the empty string, so
`{{base_url}}/style.css` becomes `/style.css` — correct for a root-hosted site.
With `--base-url /my-repo`, it becomes `/my-repo/style.css`.

Two things to keep in mind:

- `{{base_url}}` affects **only the template**, not links written inside
  Markdown. In Markdown, prefer *relative* links (`[next](posts/hello.html)`),
  which work no matter where the site is hosted.
- Don't add a trailing slash to `--base-url`; the template adds the `/` between
  the placeholder and the path.

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
browser to see the result. The nested post (`posts/hello-world.html`) shows off
front matter: it sets its own `title` and uses the `post.html` layout.

---

## Deploying to GitHub Pages

`ssg`'s output is plain static files, so hosting is just "upload the `dist`
folder". For GitHub Pages specifically:

1. Build the site (add `--base-url /your-repo` if it lives in a subdirectory).
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

---

## Known limitations in v0_02

- **Content links aren't rewritten** — the base URL affects only the template's
  `{{base_url}}` placeholder; absolute paths written inside Markdown are left
  as-is (use relative links in content instead).
- **No index/listing pages** — pages aren't automatically linked together; you
  write the navigation by hand.
- **No per-folder templates** — a page overrides its template via front matter,
  but there is no "every page in this folder uses that template" rule.
- **No clean builds** — the output directory is not emptied first, so deleted
  source files leave stale output behind.
- **Front matter is a tiny subset of YAML** — only scalar `key: value` lines;
  no nesting, lists, or multiline values.

These are deliberate: each is a natural next step for a future version (see
the roadmap in the changelog).
