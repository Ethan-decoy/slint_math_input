# slint-math-input

[简体中文](readme_zh.md)

A native mathematical expression input component for Rust and Slint.

The intended experience is editing typeset expressions directly, with LaTeX
import and export for exchanging mathematical content.

## Status

**Initial development. There is no usable formula editing API yet, and the crate
has not been published to crates.io.** This repository currently provides the
library skeleton, licensing, contributor instructions, and CI. Package publication
is disabled until an initial implementation is ready.

Planned capabilities:

- Structured editing of typeset mathematical expressions.
- Cursor movement, selection, and predictable edit/undo behavior.
- LaTeX import and export within a documented supported subset.
- Embedding as a Slint component in other applications.

The project is an independently maintained third-party component for Slint.

## Development

Rust 1.85 or newer is required. The current skeleton has no GUI dependencies;
native desktop prerequisites will be documented when the Slint component is added.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --no-deps
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution and verification details.

## License

Licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
