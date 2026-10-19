//! Clipboard transactions tested without touching the user's system clipboard.
#![cfg(feature = "slint-ui")]
use slint_math_input::{
    Action, Direction, Error,
    gui::{ClipboardAccess, MathInputController},
};
use std::{cell::RefCell, rc::Rc};
struct MemoryClipboard(Rc<RefCell<String>>);
impl ClipboardAccess for MemoryClipboard {
    fn read_text(&mut self) -> Result<String, Error> {
        Ok(self.0.borrow().clone())
    }
    fn write_text(&mut self, text: String) -> Result<(), Error> {
        *self.0.borrow_mut() = text;
        Ok(())
    }
}
#[test]
fn copy_cut_paste_and_undo_preserve_structural_fragments() {
    let clipboard = Rc::new(RefCell::new(String::new()));
    let mut controller = MathInputController::with_clipboard(MemoryClipboard(clipboard.clone()));
    controller.insert("1/2");
    controller.editor.apply(Action::SelectAll).unwrap();
    assert!(controller.key("c", true, false, false));
    assert_eq!(&*clipboard.borrow(), r"\frac{1}{2}");
    assert!(controller.key("x", true, false, false));
    assert_eq!(controller.editor.latex_preview(), "");
    assert!(controller.key("v", true, false, false));
    assert_eq!(controller.editor.latex().unwrap(), r"\frac{1}{2}");
    assert!(controller.key("z", true, false, false));
    assert_eq!(controller.editor.latex_preview(), "");
    assert!(controller.key("z", true, true, false));
    assert_eq!(controller.editor.latex().unwrap(), r"\frac{1}{2}");
}
#[test]
fn collapsed_shift_selection_cut_does_not_delete_an_adjacent_atom() {
    let clipboard = Rc::new(RefCell::new(String::new()));
    let mut controller = MathInputController::with_clipboard(MemoryClipboard(clipboard));
    controller.insert("abc");
    controller
        .editor
        .apply(Action::Move(Direction::Left, true))
        .unwrap();
    controller
        .editor
        .apply(Action::Move(Direction::Right, true))
        .unwrap();
    assert!(!controller.editor.has_selection());
    controller.key("x", true, false, false);
    assert_eq!(controller.editor.latex().unwrap(), "abc");
}
#[test]
fn unsupported_clipboard_latex_preserves_the_existing_document() {
    let clipboard = Rc::new(RefCell::new(r"\unsupported{x}".into()));
    let mut controller = MathInputController::with_clipboard(MemoryClipboard(clipboard));
    controller.insert("abc");
    let before = controller.editor.draft().unwrap();
    controller.key("v", true, false, false);
    assert!(controller.error.is_some());
    assert_eq!(controller.editor.draft().unwrap(), before);
}

#[test]
fn keypad_uses_the_same_clipboard_selection_and_failure_transactions() {
    let clipboard = Rc::new(RefCell::new(String::new()));
    let mut controller = MathInputController::with_clipboard(MemoryClipboard(clipboard.clone()));
    for token in ["1", "key:fraction", "2", "key:next", "+", "x", "key:square"] {
        controller.keypad(token, false);
        assert!(controller.error.is_none());
    }
    assert_eq!(controller.editor.latex().unwrap(), r"\frac{1}{2}+x^{2}");
    controller.keypad("key:select-all", false);
    controller.keypad("key:cut", false);
    assert_eq!(&*clipboard.borrow(), r"\frac{1}{2}+x^{2}");
    assert_eq!(controller.editor.latex_preview(), "");
    controller.keypad("key:paste", false);
    assert_eq!(controller.editor.latex().unwrap(), r"\frac{1}{2}+x^{2}");
    let before = controller.editor.draft().unwrap();
    controller.keypad("key:unknown", false);
    assert!(controller.error.is_some());
    assert_eq!(controller.editor.draft().unwrap(), before);
}

#[test]
fn clipboard_paste_resets_the_vertical_navigation_column() {
    let clipboard = Rc::new(RefCell::new(r"\frac{1}{a}".to_owned()));
    let mut controller = MathInputController::with_clipboard(MemoryClipboard(clipboard));
    controller.editor.font_size = 22.0;
    controller.insert("1/2");
    controller.keypad("key:up", false);
    controller.keypad("key:paste", false);
    assert!(controller.error.is_none());
    controller.keypad("key:up", false);
    assert_eq!(controller.editor.cursor().offset, 1);
}
