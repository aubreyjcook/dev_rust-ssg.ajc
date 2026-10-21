//! Markdown-to-HTML conversion and page-title extraction.

/// Convert a Markdown document into an HTML fragment.
///
/// `pulldown-cmark` is a *streaming* parser: rather than building a syntax
/// tree in memory, it produces an iterator of [`pulldown_cmark::Event`]
/// values (e.g. "a heading starts", "this is inline text", "a paragraph
/// ends"). [`pulldown_cmark::html::push_html`] consumes that iterator and
/// appends the corresponding HTML to a `String`.
///
/// We own the `String` that receives the HTML, which is a small but useful
/// demonstration of Rust's ownership rules: `push_html` *borrows* our
/// `&mut String` and mutates it, and when it returns we still own the fully
/// populated value.
pub fn render(markdown: &str) -> String {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

/// Extract a page title from the first Markdown heading.
///
/// We look for ATX-style headings and return the first match, preferring a
/// level-1 heading over a level-2 heading. Returning [`Option`] lets the
/// caller decide what to do when a document has no heading at all (it falls
/// back to the file name).
///
/// # Examples
///
/// ```
/// use ssg::markdown::extract_title;
///
/// assert_eq!(extract_title("# Home\n\ntext"), Some("Home".to_string()));
/// assert_eq!(extract_title("intro\n## Getting started"), Some("Getting started".to_string()));
/// assert_eq!(extract_title("no headings here"), None);
/// ```
pub fn extract_title(markdown: &str) -> Option<String> {
    let mut h2: Option<String> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();

        // `strip_prefix` returns the rest of the string when it starts with
        // the given prefix, and `None` otherwise. Note that `"## Title"`
        // does *not* start with `"# "` (its second character is `#`, not a
        // space), so this check matches exactly one `#`.
        if let Some(rest) = trimmed.strip_prefix("# ") {
            return Some(rest.trim().to_string());
        }

        // Remember the first level-2 heading as a fallback, but keep looking
        // in case a level-1 heading appears later.
        if h2.is_none() {
            if let Some(rest) = trimmed.strip_prefix("## ") {
                h2 = Some(rest.trim().to_string());
            }
        }
    }

    h2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_level_one_heading() {
        let src = "## Sub first\n# Main title\n";
        assert_eq!(extract_title(src), Some("Main title".to_string()));
    }

    #[test]
    fn falls_back_to_level_two_heading() {
        assert_eq!(
            extract_title("intro\n## Getting started"),
            Some("Getting started".to_string())
        );
    }

    #[test]
    fn returns_none_without_headings() {
        assert_eq!(extract_title("just some **text**"), None);
    }

    #[test]
    fn renders_basic_markdown() {
        let html = render("# Hi");
        assert!(html.contains("<h1>Hi</h1>"), "unexpected html: {html}");
    }
}
