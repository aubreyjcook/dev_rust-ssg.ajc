---
title: A friendly greeting
template: post.html
---
This page is rendered by the **`post.html`** layout because its front matter
sets `template: post.html`.

Its title comes from the `title:` field in the front matter — this document
has no level-1 heading of its own, so the heading-based title extraction has
nothing to find and the front matter wins.

## A subsection

- Markdown lists become HTML lists.
- *Emphasis* and **strong** text are supported.
- Inline `code` and [links](https://commonmark.org) work too.
