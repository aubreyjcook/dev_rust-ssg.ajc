# Home

Welcome to the **ssg** example site.

This page started life as a Markdown file and was turned into the HTML you
are reading now. Try the links in the header, or visit
[the nested post](posts/hello-world.html).

## What you're seeing

- **Markdown in, HTML out** — this file is `content/index.md`.
- **A shared template** — the header, footer and stylesheet come from
  `template.html`.
- **A base URL** — every asset path in the template goes through
  `{{base_url}}`, so the site still works when hosted in a subdirectory.
- **Front matter** — the nested post carries a `title:` and a `template:` in
  its front matter, overriding the defaults for that one page.
- **Assets copied through** — `style.css` is copied verbatim from `content/`.
