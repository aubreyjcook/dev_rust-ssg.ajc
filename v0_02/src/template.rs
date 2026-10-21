//! Page templates via simple `{{placeholder}}` substitution.
//!
//! The template "language" is intentionally tiny: a template is an ordinary
//! HTML file that may contain these placeholders:
//!
//! - `{{title}}`   — required; replaced with the page's title
//! - `{{content}}` — required; replaced with the generated HTML body
//! - `{{base_url}}` — optional; replaced with the configured base URL (often
//!   empty), used to prefix asset paths for sites in a subdirectory
//!
//! Anything else in the file is copied through verbatim. Keeping the engine
//! to a couple of `str::replace` calls means there is nothing clever to
//! debug, and the whole feature fits on one screen.

use std::path::Path;

use crate::error::{Result, SsgError};

/// A page template loaded from disk.
///
/// The struct holds the raw template text privately, so the only way to use
/// it is through [`Template::load`] and [`Template::render`]. Encapsulating
/// the text behind those methods lets `load` *validate* the template up
/// front and reject a broken one before any files are written.
pub struct Template {
    source: String,
}

impl Template {
    /// Read a template from `path` and check it contains both placeholders.
    pub fn load(path: &Path) -> Result<Self> {
        let source = std::fs::read_to_string(path).map_err(|err| {
            // Distinguish "file missing" (a user error we can explain) from
            // any other i/o problem.
            if err.kind() == std::io::ErrorKind::NotFound {
                SsgError::NotFound(path.display().to_string())
            } else {
                SsgError::Io(err)
            }
        })?;

        // Validate early: if a placeholder is missing we'd rather fail now
        // than silently emit pages with literal `{{content}}` in them.
        for placeholder in ["{{title}}", "{{content}}"] {
            if !source.contains(placeholder) {
                return Err(SsgError::MissingPlaceholder(placeholder.to_string()));
            }
        }

        Ok(Template { source })
    }

    /// Render a complete page by substituting the title, HTML content, and
    /// base URL.
    ///
    /// `base_url` may be empty (the common case). Replacing `{{base_url}}`
    /// with an empty string is what makes `href="{{base_url}}/style.css"`
    /// collapse to `href="/style.css"` when the site lives at the domain
    /// root.
    pub fn render(&self, title: &str, content: &str, base_url: &str) -> String {
        // `str::replace` returns a *new* `String`; it never mutates the
        // template. That is why `render` can take `&self` (a shared
        // reference) — many pages can safely share one template.
        self.source
            .replace("{{title}}", title)
            .replace("{{content}}", content)
            .replace("{{base_url}}", base_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test module is a child of the `template` module, so it may touch
    // the private `source` field and construct a `Template` directly — that
    // lets us test `render` without any filesystem access.
    #[test]
    fn substitutes_both_placeholders() {
        let template = Template {
            source: "<h1>{{title}}</h1>{{content}}".to_string(),
        };
        assert_eq!(
            template.render("Home", "<p>hi</p>", ""),
            "<h1>Home</h1><p>hi</p>"
        );
    }

    #[test]
    fn substitutes_base_url() {
        let template = Template {
            source: "<link href=\"{{base_url}}/style.css\">".to_string(),
        };
        assert_eq!(
            template.render("", "", "/my-repo"),
            "<link href=\"/my-repo/style.css\">"
        );
        // An empty base URL collapses to a root-relative path.
        assert_eq!(template.render("", "", ""), "<link href=\"/style.css\">");
    }

    #[test]
    fn leaves_unknown_placeholders_untouched() {
        let template = Template {
            source: "{{title}} / {{author}}".to_string(),
        };
        // `{{author}}` is not a placeholder we understand, so it passes
        // through verbatim.
        assert_eq!(template.render("Home", "", ""), "Home / {{author}}");
    }
}
