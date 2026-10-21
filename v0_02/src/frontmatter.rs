//! YAML-style *front matter* parsing.
//!
//! A Markdown file may begin with a small metadata block delimited by lines
//! containing exactly `---`:
//!
//! ```text
//! ---
//! title: My page
//! template: post.html
//! ---
//! # My page
//! ...the actual Markdown body...
//! ```
//!
//! We support only a tiny subset of YAML — one scalar `key: value` pair per
//! line — and hand-roll the parser. A real project would use a YAML library
//! such as `serde_yaml`; here the goal is to keep the parsing small enough to
//! read top-to-bottom. (The roadmap in `docs/CHANGELOG.md` notes this as a
//! possible future swap.)
//!
//! # Why this function returns a tuple, not a `Result`
//!
//! [`parse`] can never "fail" in a way the caller must handle: a file with no
//! front matter is simply returned unchanged. When a function is *total*
//! (always succeeds), Rust convention is to return a plain value rather than
//! wrapping it in `Result`. Compare this with [`crate::error::SsgError`],
//! which models genuinely fallible operations.

/// Metadata extracted from a document's front matter block.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FrontMatter {
    /// Page title override; wins over any Markdown heading.
    pub title: Option<String>,
    /// Per-page template file, resolved relative to the default template.
    pub template: Option<String>,
}

/// Split `source` into `(front matter, body)`.
///
/// Front matter is recognised only when the *first* line is exactly `---`
/// **and** a later line is exactly `---` (the closing delimiter). If either
/// is missing, the text is treated as ordinary content — a lone leading
/// `---` is a horizontal rule, not an error, and the whole source is
/// returned as the body with no front matter.
pub fn parse(source: &str) -> (FrontMatter, &str) {
    // Front matter requires the very first line to be a `---` delimiter.
    // `split_once('\n')` gives us the first line and the rest of the file in
    // one step; `trim` on the first line also tolerates Windows `\r\n` line
    // endings.
    let mut rest = match source.split_once('\n') {
        Some((first, rest)) if first.trim() == "---" => rest,
        _ => return (FrontMatter::default(), source),
    };

    let mut fm = FrontMatter::default();

    // Walk the remaining lines one at a time, carrying `rest` (the unread
    // part of the file) forward so we know exactly where the body begins.
    while !rest.is_empty() {
        let (line, remaining) = match rest.split_once('\n') {
            Some((line, remaining)) => (line, remaining),
            None => (rest, ""), // last line has no trailing newline
        };

        // Closing delimiter reached: everything after it is the Markdown body.
        if line.trim() == "---" {
            return (fm, remaining);
        }

        if let Some((key, value)) = split_pair(line) {
            match key {
                "title" => fm.title = Some(value),
                "template" => fm.template = Some(value),
                // Unknown keys are ignored on purpose: it keeps the parser
                // forward-compatible (a page can carry extra metadata that
                // later versions will understand).
                _ => {}
            }
        }

        rest = remaining;
    }

    // No closing delimiter was found, so the leading `---` was just ordinary
    // content (for example a horizontal rule). Hand back the whole file.
    (FrontMatter::default(), source)
}

/// Parse one `key: value` line into `(key, value)`.
///
/// Lines that don't contain a `:` (or whose key is empty) yield `None` and
/// are skipped by the caller.
fn split_pair(line: &str) -> Option<(&str, String)> {
    // `split_once` returns `Some((before, after))` for the first `:` and
    // `None` when there is none. A value may itself contain a `:`, so we
    // split on the *first* one only.
    let (key, value) = line.split_once(':')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, unquote(value.trim()).to_string()))
}

/// Strip one pair of surrounding matching quotes, if present.
///
/// This lets front matter use `title: "Hello, world"` without the quotes
/// ending up inside the title. A value with mismatched or absent quotes is
/// returned as-is.
fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        // Safe to slice at byte offsets 1 and len-1: the characters removed
        // are ASCII quotes, which are one byte each.
        &value[1..value.len() - 1]
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_and_template() {
        let src = "---\ntitle: My page\ntemplate: post.html\n---\n# Body\n";
        let (fm, body) = parse(src);
        assert_eq!(fm.title, Some("My page".to_string()));
        assert_eq!(fm.template, Some("post.html".to_string()));
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn no_front_matter_returns_source_unchanged() {
        let src = "# Just a heading\n";
        let (fm, body) = parse(src);
        assert_eq!(fm, FrontMatter::default());
        assert_eq!(body, src);
    }

    #[test]
    fn lone_leading_dashes_are_not_front_matter() {
        // A horizontal rule at the top, but no closing delimiter.
        let src = "---\n# Heading\n";
        let (fm, body) = parse(src);
        assert_eq!(fm, FrontMatter::default());
        assert_eq!(body, src);
    }

    #[test]
    fn strips_quotes_and_ignores_unknown_keys() {
        let src = "---\ntitle: \"Quoted title\"\nauthor: Someone\n---\nbody\n";
        let (fm, _) = parse(src);
        assert_eq!(fm.title, Some("Quoted title".to_string()));
        assert_eq!(fm.template, None);
    }
}
