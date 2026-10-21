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

use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::error::Result;
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

/// Build the entire site described by `config`.
///
/// This is the public entry point the binary calls. It loads and validates
/// the template once (so a bad template fails before any output is written),
/// then recurses over the content directory.
pub fn build(config: &Config) -> Result<BuildStats> {
    let template = Template::load(&config.template_path)?;

    // Fail early with a clear message if the content directory is missing.
    if !config.content_dir.is_dir() {
        return Err(crate::error::SsgError::NotFound(
            config.content_dir.display().to_string(),
        ));
    }

    // `create_dir_all` succeeds silently if the directory already exists,
    // and creates any missing parents along the way.
    fs::create_dir_all(&config.output_dir)?;

    let mut stats = BuildStats::default();
    walk(config, &config.content_dir, &template, &mut stats)?;
    Ok(stats)
}

/// Recurse through `dir`, processing every file found.
fn walk(config: &Config, dir: &Path, template: &Template, stats: &mut BuildStats) -> Result<()> {
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
            walk(config, &path, template, stats)?;
        } else {
            process_file(config, &path, template, stats)?;
        }
    }
    Ok(())
}

/// Convert one Markdown file to HTML, or copy any other file verbatim.
fn process_file(
    config: &Config,
    path: &Path,
    template: &Template,
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

        // No heading in the document? Fall back to the file name, lightly
        // prettified ("hello-world.md" -> "Hello world").
        let title = markdown::extract_title(&source).unwrap_or_else(|| fallback_title(path));

        let body = markdown::render(&source);
        let page = template.render(&title, &body);

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
}
