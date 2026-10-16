//! Structured mathematical editing, shared layout/hit testing, and a Slint surface.
//!
//! ```
//! use slint_math_input::{Action, Direction, Editor};
//! let mut editor = Editor::new();
//! editor.apply(Action::Insert("1/2".into()))?;
//! editor.apply(Action::Move(Direction::Right, false))?;
//! assert_eq!(editor.latex()?, r"\frac{1}{2}");
//! let restored = Editor::from_draft(&editor.draft()?)?;
//! assert_eq!(restored.latex()?, editor.latex()?);
//! # Ok::<(), slint_math_input::Error>(())
//! ```

pub mod catalog;
mod editor;
mod latex;
mod layout;
mod model;
mod render;
mod storage;

#[cfg(feature = "slint-ui")]
pub mod gui;

pub use editor::{Action, Direction, Editor, Position, Template};
pub use layout::{Caret, Layout, Rect};
pub use model::{Accent, Alphabet, MatrixStyle};
pub use render::Rendered;

/// Return the installed `.slint` library entry point for a host's `build.rs`.
/// Register this path with `slint_build::CompilerConfiguration::with_library_paths`.
pub fn slint_library_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/smi.slint")
}

/// A recoverable editing, import, draft, or rendering error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// A message suitable for an editor's diagnostic area.
    pub message: String,
    /// Byte position in the imported source, when available.
    pub offset: Option<usize>,
}
impl Error {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset: None,
        }
    }
    pub(crate) fn at(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset: Some(offset),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{} (byte {offset})", self.message),
            None => f.write_str(&self.message),
        }
    }
}
impl std::error::Error for Error {}
