# Changelog

All notable user-facing changes are recorded here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Conventional Commit and pull request title checks, with contributor guidance.
- Bug report, feature request, and integration question forms with triage rules.
- Pull request and release templates, categorized generated notes, and release policy.

### Fixed

- Keep typed operators in the active superscript or subscript instead of
  silently moving the caret to the parent row; keyboard and keypad agree.
- Exclude generated Python bytecode from source packages after repository checks.
- Reset the remembered vertical-navigation column after imports, pastes, edits,
  and explicit cursor changes; preserve it when an edit is rejected.

## [0.1.0-preview.1] - 2026-10-16

### Added

- Structured mathematical editing, shared layout, rendering and pointer geometry;
  structural selection, replacement, deletion, clipboard exchange, and undo/redo.
- Fractions, roots, scripts, scalable and ghost delimiters, centered limits,
  derivatives, binomials, accents, annotations, operators, and mathematical alphabets.
- Matrix styles and safe resizing, aligned equations, and piecewise expressions.
- A 319-entry core notation catalog and a mouse keypad with numeric, alphabetic,
  and function pages; nine directly selectable categories expose 303 menu entries.
- Versioned JSON drafts, validated restoration, file persistence, and scoped
  LaTeX import/export that identifies unfinished expressions.
- Independent `MathInput` and `MathKeypad` components, combined `MathEditor`,
  host-independent Rust controller, and `ui/smi.slint` entry point.
- A maintained default window template, Rust wiring example, integration guide,
  behavior regression tests, and offscreen Slint integration checks.
- STIX Two Math with its license, MIT OR Apache-2.0 project licensing,
  deterministic CJK test-font fixtures, and desktop build prerequisites.

### Changed

- Keep window creation and application wiring in `templates/` and `examples/support/`.
- Allow hosts to select a Slint backend with `slint-ui`; retain a default
  desktop solution through `desktop`. Require Rust 1.92 and Slint 1.18.1.
- Show one function category at a time, reset its scroll on selection, and
  preserve document state and input focus during keypad navigation.
- Hide font-formatting, underline, and duplicate-alias buttons from the mouse
  menu while preserving the corresponding core notation actions.

### Fixed

- Keep keypad labels legible in light/dark hosts and across button states.
- Render mathematical operators missing from system UI fonts using the bundled font.
- Preserve standard LaTeX commands, distinct Greek variants, and optional operator bounds.

This is a GitHub source preview, not a crates.io publication. See the
[support table](docs/support.md) for verified behavior and remaining native checks.

[Unreleased]: https://github.com/Ethan-decoy/slint_math_input/compare/v0.1.0-preview.1...HEAD
[0.1.0-preview.1]: https://github.com/Ethan-decoy/slint_math_input/releases/tag/v0.1.0-preview.1
