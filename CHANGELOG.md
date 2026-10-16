# Changelog

## 0.1.0-preview.1

- Deliver the input, mathematical keypad and complete default window as three
  composable parts; add the ready-to-embed `MathEditor` and standard `ui/smi.slint` entry point.
- Move window generation and application wiring out of the library controller
  into `templates/default-window.slint` and `examples/support/`.
- Let embedded hosts choose their Slint backend with `slint-ui`; retain the default
  desktop solution through the `desktop` feature.
- Include the catalog generator in `tools/`; exclude private diagnostic examples
  from the source package while retaining the full default window and tests.

- Replace the long function list with nine fixed category buttons and one active
  category at a time; reset scrolling on category selection and retain input focus.
- Remove font-formatting, underline and duplicate-alias buttons from the mouse menu
  while preserving the corresponding notation actions in the core.
- Expand the math catalog to 319 entries in 15 topic groups, including complete
  common trigonometric families, statistics/probability names, symbols and templates.
- Add centered limits, derivative templates, binomials, floor/ceiling/norm/angle
  delimiters, accents, annotations, mathematical alphabets and explicit limit placement.
- Add matrix frame styles, aligned equations, piecewise cases and safe row/column resizing.
- Separate standard LaTeX commands from application operator names, preserve Greek
  variants and support optional integral bounds.

- Add a reusable mouse math keypad with numeric, alphabetic and function pages,
  focus restoration, selection, clipboard controls and a single-step square template.
- Keep keypad labels and button states readable in both light and dark host palettes;
  use a matching light widget palette for the demo.
- Embed the mathematical font in the function catalog to display operators missing
  from system UI fonts, with rendered-label coverage for the entire catalog.
- Add the structured editing core, mathematical layout and shared pointer geometry.
- Add fractions, roots, scripts, ghost delimiters, functions, bounded operators and matrices.
- Add validated JSON drafts, file persistence, scoped LaTeX interchange and undo/redo.
- Add the reusable Slint input, native demo, clipboard adapter and host integration guide.
- Add editing replays, rendering examples, benchmarks and offscreen Slint integration tests.
- Configure contributor checks and CI for the implementation; require Rust 1.92.
- Supply deterministic CJK test-font fixtures and Linux build prerequisites;
  retain full label-visibility assertions and current Clippy checks.
- Adopt MIT OR Apache-2.0 licensing.

This preview is a GitHub source release and is not published to crates.io.
See the support table for verification coverage, native platform checks and
differences from Desmos.
