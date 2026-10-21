//! End-to-end mathematical editing and serialization behavior.
use slint_math_input::{Action, Direction, Editor, Template};

fn type_text(editor: &mut Editor, value: &str) {
    editor.apply(Action::Insert(value.into())).unwrap();
}
fn move_to(editor: &mut Editor, direction: Direction) {
    editor.apply(Action::Move(direction, false)).unwrap();
}

#[test]
fn incomplete_fraction_can_be_saved_restored_and_completed() {
    let mut editor = Editor::new();
    type_text(&mut editor, "1/");
    assert!(editor.latex().is_err());
    let cursor = editor.cursor();
    let mut restored = Editor::from_draft(&editor.draft().unwrap()).unwrap();
    assert_eq!(restored.cursor(), cursor);
    type_text(&mut restored, "2");
    assert_eq!(restored.latex().unwrap(), r"\frac{1}{2}");
    move_to(&mut restored, Direction::Up);
    assert_ne!(restored.cursor().row, cursor.row);
    move_to(&mut restored, Direction::Down);
    assert_eq!(
        restored.cursor(),
        slint_math_input::Position {
            offset: 1,
            ..cursor
        }
    );
}

#[test]
fn slash_captures_a_term_and_horizontal_arrows_traverse_both_rows() {
    let mut editor = Editor::new();
    type_text(&mut editor, "a+b/2");
    move_to(&mut editor, Direction::Right);
    assert_eq!(editor.latex().unwrap(), r"a+\frac{b}{2}");
    move_to(&mut editor, Direction::Left);
    type_text(&mut editor, "c");
    assert_eq!(editor.latex().unwrap(), r"a+\frac{b}{2c}");
    editor.apply(Action::Home(false)).unwrap();
    move_to(&mut editor, Direction::Left);
    type_text(&mut editor, "d");
    assert_eq!(editor.latex().unwrap(), r"a+\frac{bd}{2c}");
    move_to(&mut editor, Direction::Right);
    type_text(&mut editor, "e");
    assert_eq!(editor.latex().unwrap(), r"a+\frac{bd}{e2c}");
}

#[test]
fn negative_exponents_keep_arithmetic_inside_until_explicit_exit() {
    let mut editor = Editor::new();
    type_text(&mut editor, "x^-2+1");
    assert_eq!(editor.latex().unwrap(), "x^{-2+1}");
    editor.apply(Action::Undo).unwrap();
    assert_eq!(editor.latex_preview(), "");
    editor.apply(Action::Redo).unwrap();
    assert_eq!(editor.latex().unwrap(), "x^{-2+1}");
    editor.apply(Action::Escape(Direction::Right)).unwrap();
    type_text(&mut editor, "+q");
    assert_eq!(editor.latex().unwrap(), "x^{-2+1}+q");
}

#[test]
fn square_wraps_a_selection_and_is_one_undoable_operation() {
    let mut editor = Editor::new();
    editor.apply(Action::Template(Template::Square)).unwrap();
    assert_eq!(editor.latex_preview(), "");
    type_text(&mut editor, "x+1");
    editor.apply(Action::SelectAll).unwrap();
    let before = editor.draft().unwrap();
    editor.apply(Action::Template(Template::Square)).unwrap();
    assert_eq!(editor.latex().unwrap(), "{x+1}^{2}");
    assert!(editor.at_root());
    editor.apply(Action::Undo).unwrap();
    assert_eq!(editor.draft().unwrap(), before);
    editor.apply(Action::Redo).unwrap();
    type_text(&mut editor, "+2");
    assert_eq!(editor.latex().unwrap(), "{x+1}^{2}+2");
}

#[test]
fn paired_brackets_preserve_ghost_state_in_drafts() {
    let mut editor = Editor::new();
    type_text(&mut editor, "(x+1");
    assert!(editor.latex().is_err());
    let mut restored = Editor::from_draft(&editor.draft().unwrap()).unwrap();
    type_text(&mut restored, ")");
    assert_eq!(restored.latex().unwrap(), r"\left(x+1\right)");
}

#[test]
fn selection_replacement_and_undo_restore_content_and_selection() {
    let mut editor = Editor::new();
    type_text(&mut editor, "abc");
    editor.apply(Action::Move(Direction::Left, true)).unwrap();
    assert_eq!(editor.copy_latex(), "c");
    let draft = editor.draft().unwrap();
    type_text(&mut editor, "z");
    assert_eq!(editor.latex().unwrap(), "abz");
    editor.apply(Action::Undo).unwrap();
    assert_eq!(editor.draft().unwrap(), draft);
}

#[test]
fn failed_import_is_atomic_and_has_a_source_location() {
    let mut editor = Editor::new();
    type_text(&mut editor, "x+1");
    let before = editor.draft().unwrap();
    let error = editor.import_latex(r"\unsupported{x}").unwrap_err();
    assert_eq!(error.offset, Some(0));
    assert_eq!(editor.draft().unwrap(), before);
    assert!(editor.paste(r"\unknown{lost}").is_err());
    assert_eq!(editor.draft().unwrap(), before);
}

#[test]
fn supported_latex_round_trips_all_structures() {
    for formula in [
        r"\frac{1+\sqrt{x}}{x^{2}+1}",
        r"x_{n}^{2}+\alpha+\pi",
        r"\sqrt[3]{\frac{a}{b}}",
        r"\sum_{n=1}^{10}n^{2}",
        r"\int_{0}^{1}x\,dx",
        r"\left|x+1\right|",
        r"\begin{bmatrix}1&2\\3&4\end{bmatrix}",
        r"\text{velocity }v=\frac{d}{t}",
        r"\sin\left(x\right)+\operatorname{custom}(y)",
        r"{ab}^{2}+{x^{2}}^{3}",
        r"\left[0,\frac{\pi}{2}\right)",
    ] {
        let mut editor = Editor::new();
        editor
            .import_latex(formula)
            .unwrap_or_else(|e| panic!("{formula}: {e}"));
        let canonical = editor.latex().unwrap();
        let mut again = Editor::new();
        again
            .import_latex(&canonical)
            .unwrap_or_else(|e| panic!("{canonical}: {e}"));
        assert_eq!(again.latex().unwrap(), canonical);
        assert_eq!(
            Editor::from_draft(&editor.draft().unwrap())
                .unwrap()
                .draft()
                .unwrap(),
            editor.draft().unwrap()
        );
        let image = editor.render(1.5).unwrap();
        assert!(image.pixels.as_chunks::<4>().0.iter().any(|p| p[3] != 0));
        assert_eq!(
            image.pixels.len(),
            image.pixel_width as usize * image.pixel_height as usize * 4
        );
    }
}

#[test]
fn grouped_script_bases_and_literal_text_survive_latex_exchange() {
    let mut editor = Editor::new();
    editor.import_latex(r"{ab}^{2}").unwrap();
    assert_eq!(editor.latex().unwrap(), "{ab}^{2}");
    let literal = r"50% of #1 & x_2 $ ^ ~ \ { }";
    editor.apply(Action::Clear).unwrap();
    editor
        .apply(Action::Template(Template::Text(literal.into())))
        .unwrap();
    let canonical = editor.latex().unwrap();
    let mut restored = Editor::new();
    restored.import_latex(&canonical).unwrap();
    assert_eq!(restored.latex().unwrap(), canonical);
    assert_eq!(restored.spoken_text(), literal);
    let mut typed = Editor::new();
    type_text(&mut typed, r"\sin");
    assert_eq!(typed.latex().unwrap(), r"\sin");
}

#[test]
fn malformed_drafts_and_resource_exhaustion_are_rejected() {
    let editor = Editor::new();
    let draft = editor.draft().unwrap();
    assert!(Editor::from_draft(&draft.replace("\"version\":1", "\"version\":99")).is_err());
    assert!(Editor::from_draft(&draft.replace("\"offset\":0", "\"offset\":900")).is_err());
    assert!(Editor::from_draft(&draft.replace("\"next_id\":1", "\"next_id\":0")).is_err());
    let mut editor = editor;
    assert!(
        editor
            .apply(Action::Template(Template::Matrix {
                rows: 100,
                columns: 100
            }))
            .is_err()
    );
    assert_eq!(editor.draft().unwrap(), draft);
}

#[test]
fn pointer_uses_the_rendered_fraction_geometry_at_multiple_sizes() {
    for size in [18.0, 32.0, 60.0] {
        let mut editor = Editor::new();
        editor.font_size = size;
        type_text(&mut editor, "12/34");
        let position = editor.cursor();
        let layout = editor.layout();
        let caret = layout.caret(position).unwrap();
        let x = caret.rect.x;
        let y = caret.rect.y + caret.rect.height / 2.0;
        editor.apply(Action::Home(false)).unwrap();
        editor.pointer(x, y, false).unwrap();
        assert_eq!(editor.cursor(), position);
    }
}

#[test]
fn operation_replay_preserves_invariants_and_serialization() {
    let mut editor = Editor::new();
    let operations = [
        Action::Insert("x".into()),
        Action::Insert("1/".into()),
        Action::Insert("2".into()),
        Action::Move(Direction::Left, false),
        Action::Move(Direction::Right, false),
        Action::Move(Direction::Up, false),
        Action::Move(Direction::Down, true),
        Action::Backspace,
        Action::Delete,
        Action::Undo,
        Action::Redo,
        Action::Home(false),
        Action::End(true),
        Action::Template(Template::Root),
        Action::Template(Template::Power),
        Action::Insert("(a)".into()),
        Action::Clear,
    ];
    let mut random = 741_u64;
    for step in 0..1500 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let operation = operations[(random >> 32) as usize % operations.len()].clone();
        let _ = editor.apply(operation);
        editor
            .validate()
            .unwrap_or_else(|e| panic!("step {step}: {e}"));
        if step % 50 == 0 {
            let draft = editor.draft().unwrap();
            assert_eq!(Editor::from_draft(&draft).unwrap().draft().unwrap(), draft);
        }
    }
}
