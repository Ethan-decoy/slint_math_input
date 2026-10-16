# Contributing

Issues and pull requests are welcome in this public repository. Explain the
expected editing behavior and include a small reproduction for a bug report.

## Build and checks

Use Rust 1.85 or newer with Cargo, rustfmt, and Clippy. On Windows, Rust's MSVC
toolchain requires the Visual C++ build tools and Windows SDK. The initial
skeleton does not open a desktop window or depend on Slint yet.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --no-deps
cargo package --locked
```

CI checks the current library on Linux, Windows, and macOS, and checks the
declared minimum Rust version. These checks cover the skeleton; platform GUI
behavior will need its own tests once implemented.

Add behavior-focused tests with new functionality. Use a rustdoc example when an
API becomes usable, and keep examples accurate about supported LaTeX. Add an
`examples/` or `tests/` directory when there is runnable content for it.

## Maintainer workflow

The maintainer integrates reviewed changes in a private development repository
and exports selected, committed snapshots into this repository. Public history
keeps normal incremental commits. Contributions received here are integrated
before the next export.

Package publication and version tags are separate from updating the repository.
The package currently has `publish = false`.

## Contributions and licensing

Unless you explicitly state otherwise, a contribution intentionally submitted
for inclusion in this project is licensed under both MIT and Apache-2.0, without
additional terms or conditions.
