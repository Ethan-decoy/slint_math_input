//! Structural editing rules grounded in page observations and pinned DesQuill source.
use slint_math_input::{Action, Direction, Editor};

fn insert(editor: &mut Editor, text: &str) {
    editor.apply(Action::Insert(text.into())).unwrap();
}
fn action(editor: &mut Editor, action: Action) {
    editor.apply(action).unwrap();
}

#[test]
fn denominator_boundary_backspace_removes_bar_and_preserves_both_rows() {
    let mut editor = Editor::new();
    insert(&mut editor, "12/34");
    action(&mut editor, Action::Home(false));
    action(&mut editor, Action::Backspace);
    assert_eq!(editor.latex().unwrap(), "1234");
    assert_eq!(editor.cursor().offset, 2);
    action(&mut editor, Action::Undo);
    assert_eq!(editor.latex().unwrap(), r"\frac{12}{34}");
}
#[test]
fn backspace_from_after_a_fraction_enters_its_denominator() {
    let mut editor = Editor::new();
    insert(&mut editor, "1/23");
    action(&mut editor, Action::Escape(Direction::Right));
    action(&mut editor, Action::Backspace);
    action(&mut editor, Action::Backspace);
    assert_eq!(editor.latex().unwrap(), r"\frac{1}{2}");
}

#[test]
fn desmos_keypad_trace_traverses_and_unwraps_a_fraction_without_losing_the_denominator() {
    let mut editor = Editor::new();
    insert(&mut editor, "1/2");
    for direction in [
        Direction::Right,
        Direction::Left,
        Direction::Left,
        Direction::Left,
    ] {
        action(&mut editor, Action::Move(direction, false));
    }
    action(&mut editor, Action::Backspace);
    assert_eq!(editor.latex_preview(), r"\frac{}{2}");
    action(&mut editor, Action::Backspace);
    assert_eq!(editor.latex().unwrap(), "2");
    assert_eq!(
        editor.cursor(),
        slint_math_input::Position { row: 0, offset: 0 }
    );
    action(&mut editor, Action::Undo);
    assert_eq!(editor.latex_preview(), r"\frac{}{2}");
}
#[test]
fn opening_bracket_captures_the_right_tail_and_closing_releases_the_remainder() {
    let mut editor = Editor::new();
    insert(&mut editor, "abcd");
    action(&mut editor, Action::Home(false));
    action(&mut editor, Action::Move(Direction::Right, false));
    insert(&mut editor, "(");
    assert_eq!(editor.latex_preview(), r"a\left(bcd\right.");
    action(&mut editor, Action::Move(Direction::Right, false));
    insert(&mut editor, ")");
    assert_eq!(editor.latex().unwrap(), r"a\left(b\right)cd");
}
#[test]
fn right_bracket_can_be_completed_by_its_left_partner() {
    let mut editor = Editor::new();
    insert(&mut editor, "x+1)");
    assert!(editor.latex().is_err());
    let saved = editor.draft().unwrap();
    let mut editor = Editor::from_draft(&saved).unwrap();
    action(&mut editor, Action::Home(false));
    insert(&mut editor, "(");
    assert_eq!(editor.latex().unwrap(), r"\left(x+1\right)");
}
#[test]
fn inequality_backspace_restores_the_strict_operator() {
    let mut editor = Editor::new();
    insert(&mut editor, "x<=");
    action(&mut editor, Action::Backspace);
    assert_eq!(editor.latex().unwrap(), "x<");
}
#[test]
fn typing_extends_function_names_and_prefers_the_longest_operator() {
    for (text, expected) in [("sinh", r"\sinh"), ("arcsin", r"\arcsin")] {
        let mut editor = Editor::new();
        insert(&mut editor, text);
        assert_eq!(editor.latex().unwrap(), expected);
    }
}
#[test]
fn entering_an_existing_script_does_not_duplicate_the_selected_fragment() {
    let mut editor = Editor::new();
    insert(&mut editor, "x^2");
    action(&mut editor, Action::Escape(Direction::Right));
    insert(&mut editor, "ab");
    action(&mut editor, Action::Move(Direction::Left, true));
    action(&mut editor, Action::Move(Direction::Left, true));
    insert(&mut editor, "^");
    assert_eq!(editor.latex().unwrap(), "x^{2ab}");
}

#[test]
fn plain_piecewise_and_list_paste_keep_their_visible_delimiters() {
    for (text, expected) in [
        ("{x<0:-x,x>=0:x}", r"\left\{x<0:-x,x\ge 0:x\right\}"),
        ("[1,2,3]", r"\left[1,2,3\right]"),
    ] {
        let mut editor = Editor::new();
        editor.paste(text).unwrap();
        assert_eq!(editor.latex().unwrap(), expected);
    }
}

#[test]
fn matrix_navigation_visits_adjacent_cells_and_preserves_columns() {
    let mut editor = Editor::new();
    editor
        .import_latex(r"\begin{bmatrix}1&2\\3&4\end{bmatrix}")
        .unwrap();
    action(&mut editor, Action::Move(Direction::Left, false));
    insert(&mut editor, "x");
    assert_eq!(
        editor.latex().unwrap(),
        r"\begin{bmatrix}1&2\\3&4x\end{bmatrix}"
    );
    action(&mut editor, Action::Move(Direction::Up, false));
    insert(&mut editor, "y");
    assert_eq!(
        editor.latex().unwrap(),
        r"\begin{bmatrix}1&2y\\3&4x\end{bmatrix}"
    );
}

#[test]
fn reversing_a_cross_structure_shift_selection_collapses_without_sticking() {
    let mut editor = Editor::new();
    insert(&mut editor, "1/2");
    action(&mut editor, Action::Move(Direction::Right, true));
    assert_eq!(editor.copy_latex(), r"\frac{1}{2}");
    action(&mut editor, Action::Move(Direction::Left, true));
    assert!(!editor.has_selection());
    insert(&mut editor, "x");
    assert_eq!(editor.latex().unwrap(), r"x\frac{1}{2}");
    action(&mut editor, Action::Undo);
    assert_eq!(editor.latex().unwrap(), r"\frac{1}{2}");
}

#[test]
fn importing_a_formula_resets_the_vertical_navigation_column() {
    let mut editor = Editor::new();
    editor.font_size = 22.0;
    insert(&mut editor, "1/2");
    action(&mut editor, Action::Move(Direction::Up, false));
    editor.import_latex(r"\frac{12}{ab}").unwrap();
    action(&mut editor, Action::Move(Direction::Up, false));
    assert_eq!(
        editor.cursor().offset,
        2,
        "navigate from the new formula's end"
    );
}

#[test]
fn home_and_end_reset_the_vertical_navigation_column() {
    for end in [false, true] {
        let mut editor = Editor::new();
        insert(&mut editor, "12/34");
        if end {
            action(&mut editor, Action::Home(false));
        }
        action(&mut editor, Action::Move(Direction::Up, false));
        action(
            &mut editor,
            if end {
                Action::End(false)
            } else {
                Action::Home(false)
            },
        );
        action(&mut editor, Action::Move(Direction::Down, false));
        assert_eq!(editor.cursor().offset, if end { 2 } else { 0 });
    }
}

#[test]
fn exiting_a_structure_resets_the_vertical_navigation_column() {
    let mut editor = Editor::new();
    insert(&mut editor, "12/34");
    action(&mut editor, Action::Home(false));
    action(&mut editor, Action::Move(Direction::Up, false));
    action(&mut editor, Action::Escape(Direction::Right));
    action(&mut editor, Action::Move(Direction::Up, false));
    assert_eq!(editor.cursor().offset, 2);
}

#[test]
fn catalog_function_insertion_resets_the_vertical_navigation_column() {
    let mut editor = Editor::new();
    insert(&mut editor, "1/2");
    action(&mut editor, Action::Move(Direction::Up, false));
    editor.insert_catalog("fn-sin").unwrap();
    action(&mut editor, Action::Move(Direction::Down, false));
    assert_eq!(editor.cursor().offset, 1);
}

#[test]
fn rejected_edits_preserve_the_existing_vertical_navigation_column() {
    for rejected in 0..3 {
        let mut editor = Editor::new();
        editor.import_latex(r"\frac{12345}{1}").unwrap();
        action(&mut editor, Action::Move(Direction::Up, false));
        let numerator_end = editor.cursor();
        assert_eq!(numerator_end.offset, 5);
        action(&mut editor, Action::Move(Direction::Down, false));
        let before = editor.draft().unwrap();
        let result = match rejected {
            0 => editor.import_latex(r"\unsupported{x}"),
            1 => editor.paste(r"\unsupported{x}"),
            _ => editor.apply(Action::Insert("(".repeat(70))),
        };
        assert!(result.is_err());
        assert_eq!(editor.draft().unwrap(), before);
        action(&mut editor, Action::Move(Direction::Up, false));
        assert_eq!(editor.cursor(), numerator_end, "rejected edit {rejected}");
    }
}
