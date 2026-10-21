# Supported editing and interchange

This describes the development implementation, not a promise of complete TeX or
Desmos compatibility. Validation was performed on Windows x86_64 with Rust 1.93.0.
CI checks the source on Linux, Windows and macOS, including Rust 1.92.
These automated/offscreen checks do not replace the native desktop, input-method
and accessibility verification listed below.

## Mathematical structures

| Structure | Entry and editing | LaTeX interchange | Verification |
| --- | --- | --- | --- |
| Numbers, variables, operators | Direct typing; `*` becomes a centered dot; `<=`, `>=`, `!=` combine | Unicode and the supported named operators | Editing, deletion, rendering, drafts |
| Fractions | `/` captures the selection or the term immediately to the left; an empty capture enters the numerator, otherwise the denominator | `\frac`, `\dfrac`, `\tfrac` | Incomplete draft, term capture, arrows, boundary deletion, pointer |
| Roots | `sqrt`, `√`, square-root or indexed-root template | `\sqrt{x}`, `\sqrt[n]{x}` | Round trip, nested layout, draft |
| Superscripts and subscripts | `^`, `_`, templates; a preceding base is required for typing | `x^{2}`, `x_{n}`, combined scripts | Scope, negative exponents, existing scripts, undo |
| Square | `a²` keypad key or `Template::Square` wraps the preceding atom or selection and returns to its parent row | Grouped base and `^{2}` | Selection, one-step undo, mouse input, continuing on the baseline |
| Limits | Catalog templates for `lim`, `limsup`, `liminf`, `sup`, `inf`; editable condition centered below the name | Standard operator commands, `\limits`, `\nolimits`, starred `\operatorname` | Real mouse entry, condition editing, geometry, drafts and canonical round trip |
| Derivatives | Ordinary/partial, first/second order catalog templates; `Template::Derivative` accepts orders 1–16 and pre-fills x | Editable fractions and scripts | Selected-expression capture, undo and round trip; variables/orders are editable notation, without differentiation or linked-field semantics |
| Binomial coefficients | Two editable arguments; arrows traverse both | `\binom`, `\dbinom`, `\tbinom` | Navigation, selection, unwrap preservation, drafts and round trip |
| Parentheses, brackets, braces, absolute value | Automatic partner; ghost delimiter remains until completed; opening can capture the right-hand tail | `\left`/`\right`, including `.` for an unfinished side | Capturing, closing in the middle, right-first completion, draft |
| Floor, ceiling, norm, angle brackets | Scalable paired templates and explicit closing keys | Named delimiters, with or without `\left`/`\right` | Nested fractions, ghost drafts, canonical exchange and rendering |
| Functions and Greek symbols | Automatic names such as `sin`, `sinh`, `arcsin`, `log`, `pi`, `theta`; Unicode also accepted | Named functions and supported Greek/operator commands; `\operatorname{custom}` | Conversion, longest function name, round trip |
| Large operators | Sums/products, definite/indefinite/contour/multiple integrals, big unions/intersections; optional bounds | `\sum`, `\prod`, `\int`, `\oint`, `\iint`, `\iiint`, `\bigcup`, `\bigcap`, `\coprod`, `\bigoplus`, `\bigotimes` | Optional bounds, side integral limits, centered sum limits, explicit placement, drafts and exchange |
| Lists, sets, piecewise notation | Grouped inline forms or a two-column piecewise template | Delimiters and `cases` | Branch and condition cells align on the left; row resizing, drafts and round trip |
| Rectangular matrices and aligned equations | 1–16 rows/columns; several frames, row/column vectors and resize actions | `matrix`, `pmatrix`, `bmatrix`, `Bmatrix`, `vmatrix`, `Vmatrix`, `aligned`, `gathered`; `\substack` normalizes to a plain matrix | Cell navigation, dimensions, frame preservation and round trip; shrinking nonempty cells is rejected |
| Accents and annotations | Hat, bar, underline, vector, dot, double dot, tilde, braces, annotations above/below a base | `\hat`, `\widehat`, `\bar`, `\overline`, `\underline`, `\vec`, `\overrightarrow`, `\dot`, `\ddot`, `\tilde`, `\widetilde`, `\overbrace`, `\underbrace`, `\overset`, `\underset`, `\stackrel` | Editable children, annotation content, centered brace labels, drafts and round trip |
| Mathematical alphabets | Editable bold, blackboard, calligraphic, fraktur, italic, sans and monospace regions | `\mathbf`, `\mathbb`, `\mathcal`, `\mathfrak`, `\mathit`, `\mathsf`, `\mathtt` | Latin letters and supported digits use STIX mathematical Unicode glyphs; nested styles and source are retained |
| Literal upright text | `Template::Text` or LaTeX import creates an atomic text run | `\text{...}`, `\mathrm{...}` | Literal round trip and draft; text-run interior editing is not implemented |

The [input catalog](catalog.md) lists 303 mouse-menu entries in nine selectable
categories, plus the formatting and alias actions available only through the core.
The core supports 319 entries in total.
Metadata is shared with the core through `catalog::entries()` and `Editor::insert_catalog`.
The symbol and parser vocabularies are in `src/model.rs` and `src/latex.rs`.
Standard LaTeX operators use their commands; application names such as `mean`,
`nCr`, `floor` and `mod` export through `\operatorname`. The latter is a function
name, distinct from TeX's modular-congruence commands. Greek variants stay distinct.
Commands outside this subset return an error with a byte offset where available.
Macros, arbitrary environments/packages, colors, general font selection/shaping,
multiline text paragraphs and general TeX expansion are unsupported. Display/text
style declarations are normalized to this display-oriented input surface.
The component edits notation; it does not calculate, graph, or validate mathematical
meaning. A syntactically exportable expression may still be mathematically invalid.

## Interaction rules

- Left/right arrows traverse a fraction's numerator and denominator in document
  order. Entering from the right reaches the denominator; entering from the left
  reaches the numerator. Up/down moves directly between rows, retaining the
  preferred horizontal coordinate. This replaces an earlier assumption based on
  the old MathQuill `leftRightIntoCmdGoes = 'up'` example.
- Shift extends selection. A selection crossing structural boundaries expands to
  whole atoms in the nearest shared row. Rendering, replacement and clipboard
  export use that same normalized selection.
- Backspace after a fraction enters its denominator. Backspace at a fraction
  child's left boundary removes the fraction bar and preserves both rows. Delete
  at the corresponding right boundary follows the same preservation principle.
- Operators stay in the caret's current row, including at the end of a
  superscript or subscript: typing `P^n+m` produces `P^{n+m}`. To continue in the
  parent row, explicitly leave the script with the right arrow at its end, Tab,
  or the keypad's exit command before typing the operator. An initial minus,
  limit conditions and brace annotations also retain their scope. At the end of a
  subscript, `^` enters the same base's superscript, and conversely `_` after a
  superscript enters its subscript; `x_1^2` and `x^2_1` produce the same structure.
- Tab or Escape exits the current structure; Shift exits toward its left. At the
  root these keys are returned to the host. Home/End move within the current row;
  Ctrl/Cmd+Home/End move to document boundaries. Ctrl/Cmd+Backspace/Delete delete
  to the current row boundary. There is no word-level Ctrl+arrow navigation yet.
- Pointer placement and dragging use the same logical geometry as rendering.
  The viewport follows the active caret. Formula-surface wheel scrolling,
  touch handles, double-click word selection and mobile keyboards are not provided.
- The standard mathematical keypad has numeric, alphabetic (including case switching) and
  a function/structure catalog with a fixed category bar. Selecting a category shows
  only its entries and resets its scroll position; selecting it again returns to its
  first row. The category bar and navigation footer stay visible during mouse-wheel
  scrolling. Font-formatting controls and redundant aliases are omitted from the
  menu. Its keys reuse the same core actions; changing pages or collapsing
  it preserves the document and selection. The selection latch extends subsequent
  arrow/Home/End moves and resets after an edit. Functions insert their names;
  parentheses are available separately. The host restores focus to its active input
  after keypad actions. Submit remains a separate host intent.
- Ctrl/Cmd+A/C/X/V/Z, Shift+Z and Y implement selection, LaTeX clipboard exchange,
  undo and redo. Without a selection, Copy exports the whole formula and Cut
  leaves it in place. Unsupported pasted LaTeX preserves the existing document.
- One `Action` is one undo step. Native text commits are individual actions;
  paste is one action. History stores at most 128 content-changing states. Cursor
  movements alone do not create history entries. Restoring a draft is undoable.
- Consecutive Up/Down moves retain a preferred horizontal column. Successful edits,
  import/paste, Home/End, and structure exits reset it to the new caret position.
  Rejected edits preserve both the document and the previous navigation target.

## Drafts and LaTeX

`Editor::latex()` rejects unfilled structural slots and ghost delimiters.
`latex_preview()` and clipboard copy retain empty arguments for inspection and
interchange, but cannot preserve all editing state. `draft()` is the lossless,
versioned JSON format: content, incomplete structures, row identifiers, caret and
selection are retained. Undo history and host presentation settings are not saved.
`from_draft` validates before accepting data; `restore_draft` preserves the current
editor on failure. Unknown versions/fields, invalid positions, duplicate row IDs,
oversized documents and excessive nesting are rejected.

`save_draft_to` writes and synchronizes a temporary sibling before renaming it over
the target. A failed save reports an error and does not intentionally delete the
previous target. File permissions, filesystem rename semantics and power-loss
durability remain the host/filesystem's responsibility. The demo saves only when
**Save draft** is clicked and restores that file on the next launch.

Limits: 10,000 nodes, nesting depth 64, 2 MiB document text, 16 MiB draft input,
16×16 matrix cells, and 32 million raster pixels. Raster density must be within
0.5–4; layout font size is clamped to 12–144 logical pixels. A limit error retains
the previous valid editor state. A host should display rendering errors and keep
the draft available rather than treating a failed image as lost content.

## Native integration and remaining verification

| Area | Implemented | Verified | Remaining |
| --- | --- | --- | --- |
| Slint embedding | Independent controller, input and keypad, ready-to-embed `MathEditor`, and complete window template | Separate host with two input instances, a host-owned page embedding the combined editor, default window template, native key dispatch and pointer routing in offscreen software windows | Application-specific event-loop and platform integration |
| Mouse keypad | Reusable `MathKeypad`, three pages, nine directly selectable categories, selection, navigation, clipboard, history and submit; embedded math font for catalog labels | Real pointer dispatch in an independent host; category selection/reselection, scroll reset, fraction/square entry, physical typing and keyboard button activation, page/case changes, hide/show and isolated clipboard; rendered label contrast in light/dark host palettes and hover/pressed/checked states; all 303 visible catalog labels and their characters, with formerly blank operators also checked at 1.5/2 scale factors | Touch devices, long-press repeat and screen-reader interaction |
| Focus and submission | Native TextInput focus; Tab delegates at root; Return emits intent | Focus switches between instances, root Tab advances to the second input; Return callback; structure exit | Real desktop focus traversal with surrounding controls |
| Input methods | Slint native TextInput owns preedit; committed text reaches the model | Ordinary committed text through actual TextInput callbacks | Windows Chinese/Japanese composition, candidate confirmation, cancel and cursor relocation; Linux/macOS IME |
| Clipboard | Persistent native clipboard handle; injectable host adapter | Copy/cut/paste, failure preservation and undo using an isolated adapter | Real OS clipboard interoperability, Wayland ownership |
| Accessibility | Native editable item, configurable label, structural spoken summary with blank slots | Summary content and Slint compilation | NVDA/VoiceOver/Orca navigation, math-tree semantics and selection announcements |
| Layout and scale | Shared caret/selection/hit geometry; software-rendered STIX math | Pointer at 18/32/60 px; offscreen 1.5/2/3 scale-factor events and pointer dispatch; formula corpus and native window screenshot | Real device scaling, font fallback, high contrast and theme integration |
| Persistence | Validated JSON, undoable restore, atomic file replacement | Save/load an incomplete fraction, 64-level nesting, replace an existing file, failure preservation | Application recovery policy and long-lived schema migrations |

Rendering caches glyph bitmaps within an 8 MiB budget and quantizes their placement
to one eighth of a device pixel; logical caret/hit geometry remains unquantized.
The bundled STIX Two Math font covers common Latin, Greek and mathematical glyphs.
There is no font fallback or general Unicode shaping. Other text is preserved in
drafts but may display a missing-glyph box. Native input acceptance therefore does
not imply complete CJK text rendering support.

Desmos comparison is deliberately scoped. Page keypad clicks on the Chinese
[graphing calculator](https://www.desmos.com/calculator?lang=zh-CN) confirmed slash
capture, an editable blank denominator, fraction completion and horizontal
fraction traversal. Physical keyboard injection was unavailable in that browser
session. Other rules were informed by the official
[DesQuill source](https://github.com/desmosinc/desquill), and remain comparisons
to a configurable editor rather than proof of the website's deployed settings.
The STIX font, visible empty-slot boxes, mathematical spacing, selection policy and
limited command vocabulary differ from Desmos. This is not a pixel-identical clone.

## Reproducing checks

Run the checks in [CONTRIBUTING](../CONTRIBUTING.md). The core tests also run with
`--no-default-features`. Offscreen Slint tests exercise the separately embedded
input and keyboard, the combined editor in a host-owned page, and the complete
default window. They save `local_artifacts/rendered/demo-window.png` and
`embedded-editor.png` for visual inspection. Run `tools/generate_catalog.py --check`
to check that the generated interface and catalog documentation match the metadata.
