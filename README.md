# dev_rust-ssg.ajc

A static site generator built in Rust, intended for deploying on GitHub Pages
and similar static-hosting platforms.

The project ships in **versioned directories** (`v0_01`, `v0_02`, …) because
it is still in alpha. Each version is a complete, self-contained crate and
documents what changed from the previous version — both how the generator
works and how to use it — with the goal of helping a reader who is learning
Rust understand the code.

## Versions

| Version | Status | Summary |
| --- | --- | --- |
| [`v0_01`](v0_01) | alpha · first version | Working core: Markdown → HTML, `{{title}}`/`{{content}}` templating, directory mirroring, asset copying. |

## Documentation

Each version contains its own docs:

- **`docs/USAGE.md`** — how to install and use the generator.
- **`docs/LEARNING_RUST.md`** — the code explained as a guided Rust lesson.
- **`docs/CHANGELOG.md`** — what changed in that version and why.

Start with [`v0_01/docs/USAGE.md`](v0_01/docs/USAGE.md).

## License

[MIT](LICENSE)
