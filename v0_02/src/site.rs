//! Site building: walking the content tree and emitting the output tree.
//!
//! The core idea of a static site generator is a *tree transformation*:
//!
//! ```text
//! content/                 dist/
//!   index.md        --->     index.html
//!   about.md         --->     about.html
//!   posts/                   posts/
//!     hello.md       --->       hello.html
//!   style.css        --->     style.css   (copied verbatim)
//! ```
//!
//! Markdown files are converted and wrapped in the template; every other
//! file (CSS, images, `.nojekyll`, `CNAME`, ...) is copied byte-for-byte so
//! it ends up next to the generated HTML.
//!
//! Since v0_02, a Markdown file may carry *front matter* (see the
//! [`frontmatter`] module) that overrides the page title or selects a
//! different template for that one page.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::Result;
use crate::frontmatter::{self, FrontMatter};
use crate::markdown;
use crate::template::Template;

/// A running tally of what a build produced.
#[derive(Debug, Default)]
pub struct BuildStats {
    /// Number of Markdown files converted to HTML pages.
    pub pages: u32,
    /// Number of non-Markdown files copied verbatim.
    pub assets: u32,
}

/// A cache of page templates keyed by their filesystem path.
///
/// Templates are loaded lazily on first use and then reused, so a layout
/// shared by a hundred pages is read from disk and validated only once.
/// This is also a small, real example of the [`HashMap`] collection.
#[derive(Default)]
struct TemplateCache {
    by_path: HashMap<PathBuf, Template>,
}

impl TemplateCache {
    /// Return the template at `path`, loading it on first use.
    ///
    /// The method takes `&mut self` because it may *insert* into the map, but
    /// it returns a shared `&Template`. The returned reference borrows from
    /// the cache immutably; the caller may render with it but must not touch
    /// the cache again until that reference is dropped. The borrow checker
    /// enforces this relationship (see `docs/LEARNING_RUST.md`).
    fn get(&mut self, path: &Path) -> Result<&Template> {
        if !self.by_path.contains_key(path) {
            let template = Template::load(path)?;
            self.by_path.insert(path.to_path_buf(), template);
        }
        Ok(self.by_path.get(path).expect("template was just inserted"))
    }
}

/// Build the entire site described by `config`.
///
/// This is the public entry point the binary calls. It validates the content
/// directory and the default template up front (so a bad template fails
/// before any output is written), then recurses over the content directory.
pub fn build(config: &Config) -> Result<BuildStats> {
    // Fail early with a clear message if the content directory is missing.
    if !config.content_dir.is_dir() {
        return Err(crate::error::SsgError::NotFound(
            config.content_dir.display().to_string(),
        ));
    }

    // `create_dir_all` succeeds silently if the directory already exists,
    // and creates any missing parents along the way.
    fs::create_dir_all(&config.output_dir)?;

    let mut templates = TemplateCache::default();

    // Load and validate the default template up front. The value is cached,
    // so every page that doesn't override it reuses this single instance.
    templates.get(&config.template_path)?;

    let mut stats = BuildStats::default();
    walk(config, &config.content_dir, &mut templates, &mut stats)?;
    Ok(stats)
}

/// Recurse through `dir`, processing every file found.
fn walk(
    config: &Config,
    dir: &Path,
    templates: &mut TemplateCache,
    stats: &mut BuildStats,
) -> Result<()> {
    // `read_dir` returns an iterator over directory entries, each of which
    // is itself a `Result` (a single unreadable entry must not abort the
    // whole walk, but we still surface the error with `?`).
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Guard against the user placing the output directory *inside*
            // the content directory, which would otherwise recurse forever
            // as the build copies its own output.
            if path == config.output_dir {
                continue;
            }
            walk(config, &path, templates, stats)?;
        } else {
            process_file(config, &path, templates, stats)?;
        }
    }
    Ok(())
}

/// Convert one Markdown file to HTML, or copy any other file verbatim.
fn process_file(
    config: &Config,
    path: &Path,
    templates: &mut TemplateCache,
    stats: &mut BuildStats,
) -> Result<()> {
    // Compute the path relative to the content root so the directory
    // structure is mirrored into the output directory.
    let relative = path
        .strip_prefix(&config.content_dir)
        .expect("walk only passes paths below the content dir");
    let output_path = config.output_dir.join(relative);

    // `Path::extension` returns `Option<&OsStr>`; we compare it to "md".
    let is_markdown = path.extension().and_then(|e| e.to_str()) == Some("md");

    if is_markdown {
        let source = fs::read_to_string(path)?;

        // Split off any front matter; `body` is the Markdown that follows it.
        let (front_matter, body) = frontmatter::parse(&source);

        let template_path = choose_template_path(config, &front_matter);
        let template = templates.get(&template_path)?;
        let title = choose_title(&front_matter, body, path);

        let content_html = markdown::render(body);
        let page = template.render(&title, &content_html, &config.base_url);

        // Swap the `.md` extension for `.html`.
        let output_path = output_path.with_extension("html");
        ensure_parent(&output_path)?;
        fs::write(&output_path, page)?;
        stats.pages += 1;

        if config.verbose {
            println!("  wrote {}", output_path.display());
        }
    } else {
        ensure_parent(&output_path)?;
        fs::copy(path, &output_path)?;
        stats.assets += 1;

        if config.verbose {
            println!("copied {}", output_path.display());
        }
    }

    Ok(())
}

/// Pick the template path for one page: the front-matter override if present,
/// otherwise the site-wide default.
fn choose_template_path(config: &Config, front_matter: &FrontMatter) -> PathBuf {
    match front_matter.template.as_deref() {
        Some(name) => resolve_template(&config.template_path, name),
        None => config.template_path.clone(),
    }
}

/// Decide a page's title, in precedence order:
/// front matter, then the first Markdown heading, then the file name.
fn choose_title(front_matter: &FrontMatter, body: &str, path: &Path) -> String {
    front_matter
        .title
        .clone()
        .or_else(|| markdown::extract_title(body))
        .unwrap_or_else(|| fallback_title(path))
}

/// Resolve a template name from front matter to a concrete path.
///
/// Relative names are resolved against the directory that holds the *default*
/// template, so `template: post.html` alongside a default `template.html`
/// means "`post.html` in the same folder". Absolute paths are used as-is.
fn resolve_template(default_template_path: &Path, name: &str) -> PathBuf {
    let path = Path::new(name);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match default_template_path.parent() {
        Some(parent) => parent.join(path),
        None => path.to_path_buf(),
    }
}

/// Create the directory that will contain `path`, if it doesn't exist.
fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Derive a readable title from a file name with no Markdown heading.
fn fallback_title(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");

    // `replace` accepts a *pattern*: an array of chars means "any of these",
    // so this turns both dashes and underscores into spaces in one pass.
    let humanized = stem.replace(['-', '_'], " ");
    capitalize(&humanized)
}

/// Uppercase the first character of `s`, leaving the rest untouched.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalizes_the_first_character() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn fallback_title_humanizes_a_file_name() {
        let path = Path::new("posts/hello-world.md");
        assert_eq!(fallback_title(path), "Hello world");
    }

    #[test]
    fn resolve_template_is_relative_to_the_default_template_parent() {
        let resolved = resolve_template(Path::new("theme/default.html"), "post.html");
        assert_eq!(resolved, PathBuf::from("theme/post.html"));
    }

    #[test]
    fn resolve_template_with_no_parent_uses_the_name_alone() {
        let resolved = resolve_template(Path::new("template.html"), "post.html");
        assert_eq!(resolved, PathBuf::from("post.html"));
    }
}
