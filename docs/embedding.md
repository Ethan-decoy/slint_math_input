# Embedding the input

SMI ships the mathematical input, its keyboard, and a complete default window
template. `MathEditor` combines input and keyboard for embedding in a host-owned
container. `MathInput` and `MathKeypad` are also available individually.

The editing core has no event-loop dependency. `slint-ui` adds the controller and
clipboard adapter without selecting a window backend. The default `desktop` feature
adds Winit and software rendering for the runnable template. The host owns its
window, event loop, storage policy and business submission. No template window is
compiled into the controller module.

## Core and content

```rust
use slint_math_input::{Action, Editor};

let mut editor = Editor::new();
editor.apply(Action::Insert("1/".into()))?;
let draft = editor.draft()?; // incomplete denominator is retained
assert!(editor.latex().is_err());
let mut restored = Editor::from_draft(&draft)?;
restored.apply(Action::Insert("2".into()))?;
assert_eq!(restored.latex()?, r"\frac{1}{2}");
# Ok::<(), slint_math_input::Error>(())
```

Use `import_latex` to replace a document, `paste` to insert at its selection, and
`restore_draft` for an undoable restoration. Display `Error` to the user; its optional
offset is a byte position in imported source. Save drafts at application-defined
times with `save_draft_to`, or store the returned JSON in your own persistence layer.

## Register the Slint library

Until a package is published, depend on a local checkout. A host using `build.rs`
can register the entry point as a Slint library:

```toml
[dependencies]
slint = "=1.18.1"
slint-math-input = { path = "../slint_math_input", default-features = false, features = ["slint-ui"] }

[build-dependencies]
slint-build = "=1.18.1"
slint-math-input = { path = "../slint_math_input", default-features = false }
```

```rust,ignore
// build.rs
let libraries = std::collections::HashMap::from([
    ("math-input".into(), slint_math_input::slint_library_path()),
]);
let config = slint_build::CompilerConfiguration::new().with_library_paths(libraries);
slint_build::compile_with_config("ui/app.slint", config).unwrap();
```

```slint
import { MathEditor, MathView } from "@math-input";

export component App inherits Window {
    in-out property <MathView> formula;
    callback committed(string);
    callback key(string, bool, bool, bool) -> bool;
    callback place(float, float, bool);
    callback keypad(string, bool);
    callback submit();
    public function focus-formula() { input.focus(); }
    input := MathEditor {
        view: root.formula;
        label: "Equation";
        edited(text) => { root.committed(text); }
        command(text, control, shift, alt) => {
            return root.key(text, control, shift, alt);
        }
        seek(x, y, extend) => { root.place(x, y, extend); }
        key(token, extend) => { root.keypad(token, extend); }
        submitted => { root.submit(); }
    }
}
```

For a `slint::slint!` host in the same checkout, a relative import is sufficient.
The independent host in `tests/embedding.rs` demonstrates this with two instances.

## Connect a controller

Keep one `Rc<RefCell<MathInputController>>` per input and capture weak component
handles in callbacks. Explicitly map the controller's `ViewData` to your
host-generated `MathView`; the mapping uses only primitives and `slint::Image`:

```rust,ignore
fn refresh(ui: &App, controller: &MathInputController) -> Result<(), Error> {
    let data = controller.view(ui.window().scale_factor())?;
    ui.set_formula(MathView {
        image: data.image,
        content_width: data.width,
        content_height: data.height,
        caret_x: data.caret.x,
        caret_y: data.caret.y,
        caret_height: data.caret.height,
        font_size: data.font_size,
        accessible_text: data.spoken.into(),
    });
    Ok(())
}
```

Route `committed` to `controller.insert`, `key` to `controller.key`, `place` to
`controller.pointer` and `keypad` to `controller.keypad`, then refresh the view after
each operation. `MathEditor` restores input focus after its keypad actions. Return the boolean
from `key`: unconsumed events must reach Slint and the host. Refresh after changing
font size, restoring drafts or importing content as well. On a device scale-factor
change, request a new view at the new density. The controller caches formula pixels
and layout for cursor-only changes; caret blinking stays entirely in Slint.

`controller.error` contains the latest recoverable editing/clipboard error. A
rendering error is returned by `view`; keep the prior view and let the user save
the draft if rendering exceeds a resource limit. A host can compare draft/source
before and after a callback to emit its own content-changed event. Focus changes
are exposed by `MathInput.focus-changed` and `has-focus`.

Enter emits a submit **intent**. The host decides whether incomplete or empty
content is acceptable and whether to perform a business action. Tab/Escape are
consumed only inside a structure by the standard controller. Do not put business
submission in the text-commit callback. Native IME candidate confirmation still
requires platform verification before enabling Enter-to-submit in production.

## Arranging the mathematical keypad yourself

The keyboard is a standard SMI capability. `MathEditor` already includes it;
hosts choosing their own arrangement can import `MathInput` and `MathKeypad` from
`@math-input` and put them in a different layout:

```slint
// Inside a host component with the callbacks from the example above:
VerticalLayout {
    input := MathInput { /* connect view, edited, command and seek as above */ }
    MathKeypad {
        key(token, extend) => { root.keypad(token, extend); input.focus(); }
        submitted => { root.submit(); input.focus(); }
        focus-requested => { input.focus(); }
    }
}
```

Import `MathKeypad` and declare `callback keypad(string, bool)` on the host. Route
that callback to `controller.keypad(&token, extend)` and refresh the view just as
for a native key. Ordinary text uses the regular typing rules; commands prefixed
with `key:` perform navigation, deletion, templates, history or clipboard actions.
The component handles submission separately; do not pass `key:submit` to the
controller. An unknown reserved command reports an error without changing content.
Catalog buttons send `catalog:<id>` tokens to that same controller method. Core
hosts can inspect `catalog::entries()` or call `editor.insert_catalog(id)` directly;
each insertion is one undoable action. Metadata lives in `assets/math-catalog.json`
and the bundled Slint catalog uses the same stable identifiers.

`page` is 0 for numeric keys, 1 for letters and 2 for functions. `uppercase` changes
the letter page. `selecting` extends arrow/Home/End moves, and automatically resets
after an editing action. `expanded` collapses the keypad to its page bar. Switching
pages and collapsing never change the editor's document or selection. Keep the
active input focused after each callback, including page changes; otherwise native
typing would go to a keypad button. The independent host in `tests/keypad.rs`
demonstrates mouse input, continued physical typing and isolated clipboard access.
Page 2 has nine directly selectable categories and shows one category at a time.
`category` is the zero-based category index (0–8); invalid indices display the nearest
valid category. Changing it, or clicking the active category again, resets the list
to the first row. The category bar and navigation footer stay fixed while that
category's entries scroll. `catalog-scroll-position` exposes the logical offset
within the active category; restore `category` before restoring its scroll offset.
Category changes preserve the document, selection and undo history, and request
input focus just like page changes. Font formatting and redundant aliases are
excluded from the mouse menu; their core catalog identifiers remain supported.

The keypad also has its own interface. Hosts with several editors can route
its callbacks to the currently active controller. Buttons use native Slint focus
and accessibility semantics; the host still owns submit policy and application
clipboard policy. The demonstration shows the keypad by default and keeps its
LaTeX/JSON inspector behind the **LaTeX / 草稿** button.

The keypad uses opaque light button backgrounds, dark labels and a dark blue
selected/submit state. Its colors remain legible in hosts with either light or
dark standard-widget palettes, without changing the host palette. The demo sets
its own standard-widget palette to light to match its canvas. A customizable
keypad theme is not yet exposed.

The function catalog embeds STIX Two Math for its mathematical labels, including
multiple integrals, set operators and Greek variants. Hosts do not need to install
this font or register it themselves. Ordinary controls use the host UI font, and
Slint uses its system fallback for Chinese catalog captions.

The `MathInput` surface owns native composition and resets its hidden commit buffer after
each `edited` event. The Rust document receives committed text, not a copy of the
platform's preedit buffer. Do not independently feed key text to both the editor
and TextInput, or committed text will be duplicated.

## Clipboard, threading and presentation

`MathInputController::new` lazily opens the OS clipboard and keeps its handle
alive. Use `with_clipboard` with your own `ClipboardAccess` for embedded systems,
application clipboard policy, or isolated tests. Clipboard reads only occur on an
explicit paste shortcut. A failed cut does not delete the selected expression.

Use Slint component/image handles on the Slint UI thread. The pure `Editor` can be
owned separately and its JSON exchanged with other threads. Presentations use
logical pixels; raster density changes image resolution without changing hit
coordinates. Give the surface enough height for your expected structures; its
clipped viewport follows the caret. Colors and borders currently live in
`ui/math-input.slint`; a generalized theme API is not yet implemented.

## Complete window template

[templates/default-window.slint](../templates/default-window.slint) provides the
maintained default window. It embeds `MathEditor` using the same interface available
to other hosts. [examples/support/mod.rs](../examples/support/mod.rs) wires its
controller, source inspector, draft persistence and submit reporting; the small
[demo entry point](../examples/demo.rs) launches that solution.

Copy and adapt this wiring to replace the container, toolbars or storage policy.
For embedding, use `MathEditor` in your own window; importing it does not create
the template window or start an event loop.

See [support and verification limits](support.md) before shipping a host application.
