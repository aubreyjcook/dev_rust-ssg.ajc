# Learning Rust through `ssg` (v0_02)

This guide is written for someone who already understands programming
fundamentals — variables, functions, loops, conditionals, types — in another
language, and wants to learn Rust by reading a small, real program.

The code is deliberately tiny and has almost no dependencies, so you can read
every file top-to-bottom. The comment blocks in the source explain individual
lines; this document connects them to the **Rust ideas** they demonstrate.

---

## How to read this guide

Read the source files in this order, and use the "concepts" callouts as a
companion:

1. [`src/main.rs`](../src/main.rs) — the entry point
2. [`src/error.rs`](../src/error.rs) — errors and the `Result` type
3. [`src/config.rs`](../src/config.rs) — command-line parsing
4. [`src/frontmatter.rs`](../src/frontmatter.rs) — per-page metadata
5. [`src/markdown.rs`](../src/markdown.rs) — Markdown conversion
6. [`src/template.rs`](../src/template.rs) — templating
7. [`src/site.rs`](../src/site.rs) — walking files and building

The [`Cargo.toml`](../Cargo.toml) and [`src/lib.rs`](../src/lib.rs) are also
worth a look for how a Rust project is structured.

Run the tests as you go to watch the code actually execute:

```text
cargo test
```

---

## The big picture

Here is the whole program's data flow:

```text
command line  →  Config  →  walk content/  →  each .md file
                 (config)        │                │
                                 │                ├─ frontmatter::parse()  →  (metadata, body)
                                 │                ├─ extract_title()      →  title (or front matter)
                                 │                └─ render()             →  HTML body
                                 │                              └──────────┐
                                 │                                          ▼
                                 │          template.render(title, body, base_url)
                                 │                                          │
                                 ▼                                          ▼
                          copy other files                        write .html to dist/
```

Notice that the data flows *one way* through ordinary function calls. There is
no global state, no hidden registry, no mutable singleton. That is a Rust
habit: pass values in, get values out.

---

## 1. `main.rs` — entry point, `match`, and `Result`

Read [`src/main.rs`](../src/main.rs) first; it is only ~50 lines.

**`fn main() -> ExitCode`.** Rust's entry function is `main`. Returning
[`ExitCode`](https://doc.rust-lang.org/std/process/struct.ExitCode.html)
instead of nothing lets the program report success or failure to the shell.
Look at how the `match` on the result of `run` maps `Ok(())` to
`ExitCode::SUCCESS` and `Err(err)` to `ExitCode::FAILURE`, printing the error
to stderr with `eprintln!`.

**`env::args().skip(1).collect()`.** `env::args()` returns an *iterator* over
the command-line arguments. `.skip(1)` drops the program name. `.collect()`
gathers the rest into a `Vec<String>`. This "iterator → collect" pattern
appears everywhere in Rust.

**`match`.** Match is Rust's switch, but *exhaustive*: the compiler forces you
to handle every possible value of an enum. In `run`, every `Action` variant
(`Help`, `Version`, `Build`) must be covered or the code won't compile. This is
a theme: the compiler uses types to stop you forgetting cases.

**`?`.** In `run`, `config::parse(args)?` says "if this returns an error, stop
and return that error from this function; otherwise give me the success value."
It replaces the boilerplate of `match` + `return Err` you'd write by hand.

---

## 2. `error.rs` — enums, traits, and the `Result` type

Read [`src/error.rs`](../src/error.rs).

**Errors are values.** In Rust there are no exceptions. A function that can
fail returns a `Result<T, E>`, where `T` is the success type and `E` the error
type. The type system makes "this can fail" visible in the signature.

**`enum SsgError`.** An enum with *data-carrying variants* — `Io(io::Error)`,
`UnknownCommand(String)`, and so on. Unlike enums in C or Java, Rust enum
variants can hold values. This single enum describes every failure mode of the
whole program.

**Implementing a trait.** `impl fmt::Display for SsgError` and
`impl std::error::Error for SsgError` make our type behave like every other
error: it can be formatted with `{}`, printed, and wrapped in other errors.
A *trait* is Rust's interface: a contract of methods a type must provide.
Here `Display` requires `fmt`, and `Error` requires `source`.

**`From` and `?`.** `impl From<io::Error> for SsgError` teaches Rust how to
turn a filesystem error into our error type. That is what lets `?` work on
`io::Error` at call sites: the `?` operator automatically calls the `From`
conversion.

**Type alias.** `pub type Result<T> = std::result::Result<T, SsgError>;`
defines a shorthand so the rest of the code can write `Result<T>`.

---

## 3. `config.rs` — structs, `Option`, and ownership

Read [`src/config.rs`](../src/config.rs).

**`struct Config`.** A struct groups related data with named fields. The five
`pub` fields (a directory, a template path, an output directory, a verbosity
flag, and a base URL) are everything the builder needs. `#[derive(Debug, Clone, PartialEq, Eq)]` tells
the compiler to generate common implementations for us — here, printable
debug output, the ability to clone, and equality comparison. Deriving is how
Rust avoids the boilerplate of writing those by hand.

**`impl Default for Config`.** `Default` is a trait with one method,
`default()`. Implementing it gives us `Config::default()` and lets callers fall
back to a conventional layout (`content/`, `template.html`, `dist/`).

**`Option<T>`.** `args.first()` returns `Option<&String>`: either `Some(...)`
or `None`. There is no null in Rust; "maybe there is a value" is modeled with
`Option`. Match on it and the compiler makes you handle the `None` case.

**Ownership and borrowing.** Notice the signatures: `parse(args: &[String])`
takes a *shared reference* to a slice — it borrows the arguments without
taking ownership, so the caller still has them afterwards. `require_value`
returns `Result<String>`, an *owned* `String` that the caller now owns.
Distinguishing "borrow" (`&`) from "own" is the single most important Rust
idea; it is what lets the compiler guarantee memory safety without a garbage
collector.

---

## 4. `frontmatter.rs` — string parsing and total functions

Read [`src/frontmatter.rs`](../src/frontmatter.rs).

**What front matter is.** A block at the very top of a Markdown file, delimited
by `---` lines, carrying per-page metadata. [`frontmatter::parse`] returns the
block's fields and the rest of the file together:

```ignore
let (front_matter, body) = frontmatter::parse(&source);
```

**Returning a tuple, not a `Result`.** Unlike `Template::load`, `parse` cannot
fail in a way the caller must handle: a file without front matter is simply
returned unchanged. Rust calls a function that always succeeds *total*, and the
convention is to return a plain value (here a tuple) instead of wrapping it in
`Result`. Compare the two styles across this codebase — `Result` for fallible
i/o and parsing, plain values for total transformations.

**`split_once`.** `line.split_once(':')` returns `Option<(&str, &str)>` — the
text before and after the *first* colon, or `None` when there is none.
Splitting on the first colon only means a value may itself contain a `:` (e.g.
`title: Notes: chapter one`). This "first occurrence only" behaviour is exactly
what `split_once` is for.

**`str` methods return new slices.** The parser leans on `trim` and `split_once`
rather than mutating anything. String slices are views into the original text —
cheap to pass around, and the reason `parse` can hand back a `&str` body
without copying the whole document.

**Quoting.** `unquote` strips one pair of surrounding matching quotes, so
`title: "Hello"` yields `Hello`. Note the byte-indexed lookups (`bytes[0]` and
`bytes[len - 1]`), which are safe because the characters being removed are
ASCII quotes (one byte each).

---

## 5. `markdown.rs` — external crates and iterators

Read [`src/markdown.rs`](../src/markdown.rs).

**Using a crate.** The `pulldown-cmark` dependency declared in `Cargo.toml` is
imported simply by naming it: `pulldown_cmark::Parser`. Cargo fetched it and
made it available at compile time. This is the whole story of third-party
libraries in Rust: declare in `Cargo.toml`, use by path.

**`render`.** `Parser::new(markdown)` builds a streaming parser; `push_html`
walks its events and appends HTML into `&mut html`. Note the `&mut` — a
*mutable reference* — which lets `push_html` append to a `String` we own
without taking that `String` away from us.

**`extract_title` returns `Option<String>`.** The function returns `None` when
a document has no heading, and `Some(title)` otherwise. The caller uses
`unwrap_or_else` to supply a fallback. Compare this to returning `null` or an
empty string in other languages: `Option` makes the possibility of "no value"
explicit and impossible to ignore.

**String methods.** `str::lines`, `str::trim`, `str::strip_prefix`, and
`str::replace` are all standard-library methods on the string type. They
return *new* strings rather than mutating in place — strings in Rust are
immutable by default.

---

## 6. `template.rs` — structs, methods, and `&self`

Read [`src/template.rs`](../src/template.rs).

**Methods.** `impl Template { ... }` attaches functions to the struct. The
associated functions `load` and `render` are called as `Template::load(...)`
and `template.render(...)`. `load` is a *constructor* that returns a `Result`;
it is the only way to build a `Template`, because the `source` field is
private.

**`&self` vs `&mut self` vs `self`.** `render(&self, ...)` borrows the
template immutably — it needs to read, not change, the text. Because rendering
never mutates the template, many pages can share one `Template` safely and
cheaply. If a method needed to modify its receiver it would take `&mut self`;
if it consumed it, `self`.

**Validate early.** `load` checks that the two *required* placeholders
(`{{title}}`, `{{content}}`) exist and returns an error otherwise. This is
"fail fast": a broken template is reported before any output is written, rather
than producing subtly wrong pages. The third placeholder, `{{base_url}}`, is
optional — `render` now takes a `base_url: &str` parameter (often empty) and
substitutes it, which is what lets `href="{{base_url}}/style.css"` work both at
the domain root and in a subdirectory.

**Tests.** The `#[cfg(test)] mod tests` block at the bottom shows unit tests
living in the same file as the code they test. Note how the test module — a
child module — can reach the private `source` field. Run them with
`cargo test`.

---

## 7. `site.rs` — recursion, references, and filesystem access

Read [`src/site.rs`](../src/site.rs).

**Recursion.** `walk` calls itself for every subdirectory. Because Rust has no
special-casing for recursion, this is a plain function that takes
`&mut BuildStats` and mutates a counter as it descends. The `&mut` reference
is how one function's changes become visible to its caller without returning
and re-returning a value.

**Front matter drives per-page choices.** In `process_file`, the first thing
that happens to a Markdown source is `frontmatter::parse(&source)`. The
resulting metadata chooses the title (`choose_title`) and the template
(`choose_template_path`). Notice how these two decisions are pulled out into
small named functions, each with one responsibility.

**A `HashMap` cache.** Per-page templates are stored in a `TemplateCache`
backed by `HashMap<PathBuf, Template>`. `get(&mut self, ...)` loads a template
on first use and returns a shared `&Template` on later uses, so a layout shared
by a hundred pages is read from disk once. This is the standard "memoisation"
pattern, and it is also a nice test of the borrow checker: the method needs
`&mut self` (to insert) yet returns a shared reference into the map — read the
comment on `TemplateCache::get` for the details.

**References everywhere.** `process_file(config: &Config, path: &Path, ...)`
borrows everything — nothing is copied or cloned. Templates are stored once in
the cache and then *borrowed* by every page. This is the payoff of the
ownership system: passing a reference is as cheap as passing a pointer, with
the compiler guaranteeing the referent outlives the borrow.

**`fs`.** The standard library's `std::fs` module does the filesystem work:
`read_dir`, `read_to_string`, `create_dir_all`, `write`, `copy`. Every one of
these returns a `Result`, and every call site uses `?` to propagate errors.
There is no `try/catch`; errors flow up the return values.

**`Path` and `PathBuf`.** A `Path` is a borrowed view of a filesystem path
(like `&str`); a `PathBuf` is an owned, growable path (like `String`). Notice
the code moves between them: `output_path.with_extension("html")` builds a new
`PathBuf`, `strip_prefix` computes a relative path.

---

## Rust concepts glossary

| Concept | What it is | Where in this code |
| --- | --- | --- |
| **Ownership** | Each value has one owner; when the owner goes out of scope the value is dropped. | `Config` owns its `PathBuf`s; `Template` owns its `source` `String`. |
| **Borrowing** | Passing `&value` lends access without transferring ownership. | `template.render(...)` borrows `&self`. |
| **Mutable references** | `&mut value` allows mutation through a borrow; only one at a time. | `push_html(&mut html, ...)`; `stats: &mut BuildStats`. |
| **Lifetimes** | The compiler's proof that references don't outlive their data (mostly inferred). | Implicit in every `&Path`, `&Config`, `&str`. |
| **`Option<T>`** | A value that may be absent: `Some(v)` or `None`. | `extract_title`, `args.first()`. |
| **`Result<T, E>`** | A value that may be an error: `Ok(v)` or `Err(e)`. | Every function that touches the filesystem or CLI. |
| **Enums** | A type that is one of several variants, each optionally carrying data. | `SsgError`, `Action`. |
| **Structs** | Named, grouped data. | `Config`, `Template`, `BuildStats`, `FrontMatter`. |
| **Collections** | Standard-library data structures that own their contents. | `HashMap<PathBuf, Template>` in `site.rs`; `Vec<String>` in `main.rs`. |
| **Traits** | Contracts of behavior a type can implement. | `Display`, `Error`, `Default`, `From`. |
| **`match`** | Exhaustive branching over an enum. | `run` in `main.rs`, `parse` in `config.rs`. |
| **Iterators** | Lazy sequences operated on with `.map`, `.skip`, `.collect`, etc. | `env::args()`, `fs::read_dir`, `markdown.lines()`. |
| **`?` operator** | Early-return an error, or unwrap the success value. | `fs::read_to_string(path)?` and friends. |

---

## Suggested experiments

The best way to learn is to change the code and watch the compiler guide you.
Try, in increasing difficulty:

1. **Add a placeholder** — support `{{generator}}` in the template and render
   `"ssg"` into it. (`template.rs`, and update the validation loop.)
2. **Break a page on purpose** — remove `{{content}}` from a template and run
   `ssg build` to see the validation error in action.
3. **Add `--verbose` output** — it already exists; find where `config.verbose`
   is read and add your own message.
4. **Handle `# h1` with no space** — `extract_title` only matches `# ` with a
   space. Make it also accept `#Title` and add a test.
5. **Add a front-matter field** — support `description:` and a
   `{{description}}` placeholder end to end (`frontmatter.rs`, `template.rs`,
   and `site.rs`), with a test for each piece.
6. **Sort directory entries** — `fs::read_dir` returns entries in an
   unspecified order. Collect them into a `Vec`, sort by name, and iterate.
   (This is also a real correctness improvement.)
7. **Empty the output first** — implement the "clean build" from the roadmap:
   remove the output directory before building. Be careful and back up first.

Each of these touches one idea from the glossary, and the compiler's error
messages — along with `cargo test` — will tell you exactly what to adjust.
