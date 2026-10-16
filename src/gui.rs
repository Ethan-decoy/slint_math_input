//! Host-independent Slint input, pointer, keypad and clipboard integration.
//! Embed `ui/smi.slint`; the default window adapter lives in `examples/support/`.

use crate::{Action, Direction, Editor, Error, Rect, Template};
use slint::{Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;

/// Clipboard boundary that hosts can replace for embedded systems or tests.
pub trait ClipboardAccess {
    /// Read plain text for an explicit paste operation.
    fn read_text(&mut self) -> Result<String, Error>;
    /// Store a LaTeX fragment for copy or cut.
    fn write_text(&mut self, text: String) -> Result<(), Error>;
}
struct NativeClipboard(Option<arboard::Clipboard>);
impl NativeClipboard {
    fn handle(&mut self) -> Result<&mut arboard::Clipboard, Error> {
        if self.0.is_none() {
            self.0 =
                Some(arboard::Clipboard::new().map_err(|e| Error::new(format!("Clipboard: {e}")))?);
        }
        Ok(self.0.as_mut().expect("clipboard initialized"))
    }
}
impl ClipboardAccess for NativeClipboard {
    fn read_text(&mut self) -> Result<String, Error> {
        self.handle()?
            .get_text()
            .map_err(|e| Error::new(format!("Clipboard: {e}")))
    }
    fn write_text(&mut self, text: String) -> Result<(), Error> {
        self.handle()?
            .set_text(text)
            .map_err(|e| Error::new(format!("Clipboard: {e}")))
    }
}

/// Data consumed by a Slint math surface, independent of a particular host window.
pub struct ViewData {
    /// Formula and selection, excluding the blinking cursor.
    pub image: slint::Image,
    /// Logical image width.
    pub width: f32,
    /// Logical image height.
    pub height: f32,
    /// Active cursor geometry.
    pub caret: Rect,
    /// Current mathematical source for assistive labels and inspectors.
    pub latex: String,
    /// Structural spoken summary, separate from source syntax.
    pub spoken: String,
    /// Font size in logical pixels.
    pub font_size: f32,
}
/// Host-independent keyboard, clipboard, and rendered-view adapter.
pub struct MathInputController {
    /// The editor controlled by this surface.
    pub editor: Editor,
    /// Last recoverable host or clipboard error, if any.
    pub error: Option<Error>,
    cache: RefCell<Option<ViewCache>>,
    clipboard: Box<dyn ClipboardAccess>,
}
struct ViewCache {
    root: crate::model::Row,
    selection: Option<(u64, usize, usize)>,
    font: u32,
    density: u32,
    layout: crate::Layout,
    image: slint::Image,
}
impl Default for MathInputController {
    fn default() -> Self {
        Self::new()
    }
}
impl MathInputController {
    /// Create a controller with an empty expression.
    pub fn new() -> Self {
        Self::with_clipboard(NativeClipboard(None))
    }
    /// Create a controller with host-provided clipboard storage.
    pub fn with_clipboard(clipboard: impl ClipboardAccess + 'static) -> Self {
        Self {
            editor: Editor::new(),
            error: None,
            cache: RefCell::new(None),
            clipboard: Box::new(clipboard),
        }
    }
    /// Render the current view at the requested device pixel density.
    pub fn view(&self, density: f32) -> Result<ViewData, Error> {
        let selection = self.editor.selection_range();
        let mut cache = self.cache.borrow_mut();
        if !cache.as_ref().is_some_and(|c| {
            c.root == self.editor.state.root
                && c.selection == selection
                && c.font == self.editor.font_size.to_bits()
                && c.density == density.to_bits()
        }) {
            let rendered = self.editor.render(density)?;
            let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &rendered.pixels,
                rendered.pixel_width,
                rendered.pixel_height,
            );
            *cache = Some(ViewCache {
                root: self.editor.state.root.clone(),
                selection,
                font: self.editor.font_size.to_bits(),
                density: density.to_bits(),
                layout: rendered.layout,
                image: slint::Image::from_rgba8_premultiplied(buffer),
            });
        }
        let cached = cache.as_ref().expect("view cache populated");
        let caret = cached
            .layout
            .caret(self.editor.cursor())
            .map(|c| c.rect)
            .unwrap_or_default();
        Ok(ViewData {
            image: cached.image.clone(),
            width: cached.layout.width,
            height: cached.layout.height,
            caret,
            latex: self.editor.latex_preview(),
            spoken: self.editor.spoken_text(),
            font_size: self.editor.font_size,
        })
    }
    /// Dispatch a native Slint key. Returns true when consumed.
    /// Printable text and IME commits are handled separately through `edited`.
    pub fn key(&mut self, text: &str, control: bool, shift: bool, alt: bool) -> bool {
        if alt {
            return false;
        }
        use slint::platform::Key;
        if text == char::from(Key::Tab).to_string() || text == char::from(Key::Escape).to_string() {
            if self.editor.at_root() {
                return false;
            }
            self.error = self
                .editor
                .apply(Action::Escape(if shift {
                    Direction::Left
                } else {
                    Direction::Right
                }))
                .err();
            return true;
        }
        let action = if text == char::from(Key::LeftArrow).to_string() {
            Some(Action::Move(Direction::Left, shift))
        } else if text == char::from(Key::RightArrow).to_string() {
            Some(Action::Move(Direction::Right, shift))
        } else if text == char::from(Key::UpArrow).to_string() {
            Some(Action::Move(Direction::Up, shift))
        } else if text == char::from(Key::DownArrow).to_string() {
            Some(Action::Move(Direction::Down, shift))
        } else if text == char::from(Key::Home).to_string() {
            Some(if control {
                Action::RootHome(shift)
            } else {
                Action::Home(shift)
            })
        } else if text == char::from(Key::End).to_string() {
            Some(if control {
                Action::RootEnd(shift)
            } else {
                Action::End(shift)
            })
        } else if text == char::from(Key::Backspace).to_string() {
            Some(if control {
                Action::DeleteToBoundary(Direction::Left)
            } else {
                Action::Backspace
            })
        } else if text == char::from(Key::Delete).to_string() {
            Some(if control {
                Action::DeleteToBoundary(Direction::Right)
            } else {
                Action::Delete
            })
        } else if control {
            match text.to_ascii_lowercase().as_str() {
                "a" => Some(Action::SelectAll),
                "z" if shift => Some(Action::Redo),
                "z" => Some(Action::Undo),
                "y" => Some(Action::Redo),
                "c" | "x" => {
                    let result = self.clipboard.write_text(self.editor.copy_latex());
                    match result {
                        Ok(()) if text.eq_ignore_ascii_case("x") => {
                            self.error = None;
                            if self.editor.has_selection() {
                                self.error = self.editor.apply(Action::Delete).err();
                            }
                        }
                        Ok(()) => self.error = None,
                        Err(error) => self.error = Some(error),
                    }
                    return true;
                }
                "v" => {
                    match self.clipboard.read_text() {
                        Ok(text) => self.error = self.editor.paste(&text).err(),
                        Err(error) => self.error = Some(error),
                    }
                    return true;
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(action) = action {
            self.error = self.editor.apply(action).err();
            true
        } else {
            false
        }
    }
    /// Dispatch committed text after the native input method has finished composition.
    pub fn insert(&mut self, text: &str) {
        self.error = self.editor.apply(Action::Insert(text.into())).err();
    }
    /// Dispatch an on-screen keypad token. Literal text follows normal typing;
    /// `key:` tokens insert templates, navigate, edit, or access the clipboard.
    /// Hosts handle the keypad's separate `submitted` and `focus-requested` callbacks.
    pub fn keypad(&mut self, token: &str, extend: bool) {
        if let Some(id) = token.strip_prefix("catalog:") {
            self.error = self.editor.insert_catalog(id).err();
            return;
        }
        let action = match token {
            "key:left" => Action::Move(Direction::Left, extend),
            "key:right" => Action::Move(Direction::Right, extend),
            "key:up" => Action::Move(Direction::Up, extend),
            "key:down" => Action::Move(Direction::Down, extend),
            "key:home" => Action::Home(extend),
            "key:end" => Action::End(extend),
            "key:next" => Action::Escape(Direction::Right),
            "key:backspace" => Action::Backspace,
            "key:delete" => Action::Delete,
            "key:select-all" => Action::SelectAll,
            "key:undo" => Action::Undo,
            "key:redo" => Action::Redo,
            "key:clear" => Action::Clear,
            "key:fraction" => Action::Template(Template::Fraction),
            "key:root" => Action::Template(Template::Root),
            "key:indexed-root" => Action::Template(Template::IndexedRoot),
            "key:power" => Action::Template(Template::Power),
            "key:square" => Action::Template(Template::Square),
            "key:subscript" => Action::Template(Template::Subscript),
            "key:absolute" => Action::Template(Template::Absolute),
            "key:sum" => Action::Template(Template::Sum),
            "key:product" => Action::Template(Template::Product),
            "key:integral" => Action::Template(Template::Integral),
            "key:matrix" => Action::Template(Template::Matrix {
                rows: 2,
                columns: 2,
            }),
            "key:copy" | "key:cut" | "key:paste" => {
                self.key(
                    match token {
                        "key:copy" => "c",
                        "key:cut" => "x",
                        _ => "v",
                    },
                    true,
                    false,
                    false,
                );
                return;
            }
            _ if token.starts_with("key:") => {
                self.error = Some(Error::new(format!("Unknown keypad command: {token}")));
                return;
            }
            _ => Action::Insert(token.into()),
        };
        self.error = self.editor.apply(action).err();
    }
    /// Dispatch pointer placement or dragging.
    pub fn pointer(&mut self, x: f32, y: f32, extend: bool) {
        self.error = self.editor.pointer(x, y, extend).err();
    }
}
