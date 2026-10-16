# Contributing

Issues and pull requests are welcome in this public repository. Explain the
expected editing behavior and include a small reproduction for a bug report.

## Build and checks

Use Rust 1.92 or newer with Cargo, rustfmt, and Clippy. On Windows, Rust's MSVC
toolchain requires the Visual C++ build tools and Windows SDK. The default feature
builds the Slint/Winit surface with software rendering.

On Debian/Ubuntu, install `pkg-config`, `libfontconfig1-dev`, `libxkbcommon-dev`,
`libxkbcommon-x11-dev`, `libwayland-dev`, `libx11-dev`, `libxcb-shape0-dev` and
`libxcb-xfixes0-dev` before building the desktop features.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo doc --locked --no-deps
cargo package --locked
python tools/generate_catalog.py --check
```

CI checks the current library on Linux, Windows, and macOS, and checks the
declared minimum Rust version. Slint integration tests use an offscreen software
window and actual input dispatch. Native input-method and assistive-technology
checks remain separate; keep the support table accurate.

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

## Font fixtures

The catalog visibility tests use `tests/fixtures/noto_sans_cjk_test.otf` through
Slint's `SLINT_FONT_PATH` fallback setting. Set that variable to the fixture's
absolute path before running the desktop tests on systems without CJK fonts; CI
sets it explicitly on every platform. The small renamed subset derives from Noto
Sans CJK 2.004 under OFL-1.1, retained in `tests/fixtures/ofl_noto_sans_cjk.txt`.
It is a test asset and is not registered by the library or bundled in host apps.
To regenerate it, install FontTools 4.60.2 and follow `tools/generate_test_font.py`.
