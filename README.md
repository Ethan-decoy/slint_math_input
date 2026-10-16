# slint-math-input

[简体中文](readme_zh.md)

A native mathematical expression input component for Rust and Slint.

Edit typeset expressions directly, preserve unfinished expressions as drafts,
and exchange supported mathematics through LaTeX.

## Status

**Delivery candidate `0.1.0-preview.1`; not published to crates.io.** The editing core, shared
mathematical layout, reusable Slint surface, and native demonstration are
implemented. `publish = false` remains while interaction and platform validation
continues. The [support table](docs/support.md) records verification and differences.

Implemented capabilities:

- Fractions, roots, scripts, centered limits, derivatives, binomial coefficients,
  scalable delimiters, accents/annotations, mathematical alphabets, bounded and
  unbounded operators, matrices, aligned equations and piecewise expressions.
- Keyboard and pointer navigation, structural selections, replacement, deletion,
  clipboard exchange, and undo/redo.
- Versioned JSON drafts, validated restoration, atomic file persistence, and
  strict LaTeX export that identifies incomplete expressions.
- A reusable `.slint` component with native text input, focus, scrolling, and a
  host-independent Rust controller. See [host integration](docs/embedding.md).
- A reusable mouse keypad with numeric and alphabetic pages and a scrollable
  function/structure catalog, with selection/navigation, clipboard and undo/redo.
- A standard `MathEditor` combining the input and mathematical keypad, plus a
  complete window template. Hosts can use that layout or arrange the two parts.

SMI owns mathematical editing, consistent display/hit testing, and content exchange.
Hosts own their windows, application layout, storage policy and submission actions.
The default window is a maintained template using the same interfaces as other hosts.

The project is an independently maintained third-party component for Slint.

## Development

Rust 1.92 or newer is required. The default `desktop` feature uses Slint 1.18.1,
Winit, and software rendering. `slint-ui`
provides the controller without selecting a window backend; embedded hosts choose
their own Slint backend. Windows builds need the MSVC build tools and Windows
SDK; follow [Slint's platform setup](https://docs.slint.dev/latest/docs/slint/guide/getting-started/)
for desktop prerequisites. Disable default features to use the core independently.

```sh
cargo run --locked --example demo
cargo test --locked --no-default-features
cargo check --locked --no-default-features --features slint-ui --lib
```

The demo's **Save draft** button writes `local_artifacts/demo-draft.json`; startup
restores that file. Its JSON panel also permits explicit copying and restoration.
The keypad is visible by default. Switch between **123**, **ABC**, and **函数**;
use **选择** with arrow keys to extend a selection, and **退出** to leave a
structure. **LaTeX / 草稿** opens the source and draft inspector.
The function page has a fixed category bar for structures, common functions,
calculus, trigonometry, statistics/probability, matrices, Greek letters, relations/sets
and symbols/constants. Click a category to show its entries; scroll within that
category. The category bar and navigation footer stay available. Formatting controls
and redundant aliases are omitted from this menu; the core still supports all 319
notation actions for programmatic input and LaTeX interchange.
See the [full input catalog](docs/catalog.md) for the built-in entries.

Start with [templates/default-window.slint](templates/default-window.slint) and
[its Rust adapter](examples/support/mod.rs) for the complete solution, or import
`MathEditor` from `@math-input` in your own window. `MathInput` and `MathKeypad`
remain separately available through the same entry point.

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
The bundled STIX Two Math font has its own [SIL OFL license](assets/OFL-STIX.txt).
See [font provenance](assets/README.md). Dependencies retain their respective licenses.
