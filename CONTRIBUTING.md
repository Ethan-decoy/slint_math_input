# Contributing

Issues and pull requests are welcome in this public repository. Explain the
expected editing behavior and include a small reproduction for a bug report.

## Commits and pull requests

Use [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```text
type(scope): concise summary

Explain the problem and the resulting behavior when the title is not sufficient.

Closes #123
```

Allowed types are `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`,
`ci`, `chore`, `style`, and `revert`. Scope is optional; useful scopes include
`editor`, `layout`, `latex`, `keypad`, `slint`, `docs`, `ci`, and `release`.
Use lowercase types/scopes, keep the header within 72 characters, and omit a
final period. Separate the body with a blank line; wrap prose around 72 characters
where practical. Explain **why** and observable behavior, rather than narrating
individual edits. Prefer English for public summaries; Chinese is welcome in
discussion and detailed explanations. Language alone does not invalidate a commit.

Use `!` or a `BREAKING CHANGE:` footer for incompatible changes and document the
migration. Use real issue references, not placeholder numbers. Keep one cohesive
change per commit; separate unrelated fixes or maintenance. CI-skip directives
and unfinished `fixup!`/`squash!` messages must not enter shared project history.

```sh
python tools/check_commits.py --all
python tools/check_commits.py --range main..HEAD
python -m unittest discover -s tools -p test_check_commits.py
```

The checker audits non-merge history reachable from the selected branch; generated
merge messages are exempt. PR titles use the same header format and are checked
by the Conventions workflow. Prefer squash merging after review, using the PR title
as the final commit header. Feature-branch work-in-progress commits may be cleaned
up before integration. Do not rewrite another contributor's published work without
coordination. See [issue triage](docs/issues.md) and [version history](CHANGELOG.md).

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

## Versions and distribution

See [Releases](https://github.com/Ethan-decoy/slint_math_input/releases) and the
[changelog](CHANGELOG.md) for published source versions and upgrade information.
Updates on `main` can precede a versioned release. Distribution currently uses
GitHub source releases; the package has `publish = false`.

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
