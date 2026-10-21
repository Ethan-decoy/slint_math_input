//! Operators respect the structural caret in editable scripts.
use slint_math_input::{Action, Direction, Editor};

fn type_text(editor: &mut Editor, text: &str) {
    for ch in text.chars() {
        editor.apply(Action::Insert(ch.to_string())).unwrap();
    }
}

#[test]
fn operators_at_script_end_stay_in_the_active_row() {
    for script in ['^', '_'] {
        for operator in ['+', '-', '=', '<', '>', '*'] {
            let mut editor = Editor::new();
            type_text(&mut editor, &format!("P{script}n"));
            let row = editor.cursor().row;
            type_text(&mut editor, &operator.to_string());
            assert_eq!(editor.cursor().row, row, "{script} followed by {operator}");
            type_text(&mut editor, "m");
            let mut expected = Editor::new();
            expected
                .import_latex(&format!("P{script}{{n{operator}m}}"))
                .unwrap();
            assert_eq!(editor.latex().unwrap(), expected.latex().unwrap());
        }
    }
}

#[test]
fn script_scope_survives_undo_redo_and_explicit_exit() {
    let mut editor = Editor::new();
    type_text(&mut editor, "P^n");
    let before = editor.draft().unwrap();
    editor.apply(Action::Insert("+m".into())).unwrap();
    assert_eq!(editor.latex().unwrap(), "P^{n+m}");
    let after = editor.draft().unwrap();
    editor.apply(Action::Undo).unwrap();
    assert_eq!(editor.draft().unwrap(), before);
    editor.apply(Action::Redo).unwrap();
    assert_eq!(editor.draft().unwrap(), after);
    editor.apply(Action::Move(Direction::Right, false)).unwrap();
    assert!(editor.at_root());
    type_text(&mut editor, "+q");
    assert_eq!(editor.latex().unwrap(), "P^{n+m}+q");
}

#[test]
fn pointer_reentry_and_restored_drafts_keep_operators_in_the_script() {
    let mut editor = Editor::new();
    editor.import_latex("P^{n}").unwrap();
    editor.apply(Action::Move(Direction::Left, false)).unwrap();
    let inside = editor.cursor();
    let caret = editor.layout().caret(inside).unwrap().rect;
    editor.apply(Action::Escape(Direction::Right)).unwrap();
    assert!(editor.at_root());
    editor
        .pointer(caret.x, caret.y + caret.height / 2.0, false)
        .unwrap();
    assert_eq!(editor.cursor(), inside);
    let mut restored = Editor::from_draft(&editor.draft().unwrap()).unwrap();
    type_text(&mut restored, "+m");
    assert_eq!(restored.latex().unwrap(), "P^{n+m}");
}

#[cfg(feature = "slint-ui")]
#[test]
fn committed_text_and_keypad_both_keep_script_scope() {
    use slint_math_input::gui::MathInputController;
    for keypad in [false, true] {
        let mut controller = MathInputController::new();
        for text in ["P", if keypad { "key:power" } else { "^" }, "n", "+", "m"] {
            if keypad {
                controller.keypad(text, false);
            } else {
                controller.insert(text);
            }
            assert!(controller.error.is_none());
        }
        assert_eq!(
            controller.editor.latex().unwrap(),
            "P^{n+m}",
            "keypad={keypad}"
        );
        controller.keypad("key:next", false);
        controller.keypad("+", false);
        controller.keypad("q", false);
        assert_eq!(controller.editor.latex().unwrap(), "P^{n+m}+q");
    }
}
