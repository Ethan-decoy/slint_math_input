//! User-level continuous typing: command expansion, fields, grouping and history.
use slint_math_input::{Action, Direction, Editor, Template};

fn editor() -> Editor {
    let mut editor = Editor::new();
    editor.structured_typing = true;
    editor
}
fn typed(editor: &mut Editor, text: &str) {
    for ch in text.chars() {
        editor.apply(Action::Insert(ch.to_string())).unwrap();
        editor.validate().unwrap();
    }
}

#[test]
fn logarithm_typing_and_catalog_share_base_and_argument_fields() {
    for entry in [None, Some("log-base"), Some("fn-log")] {
        let mut editor = editor();
        if let Some(entry) = entry {
            editor.insert_catalog(entry).unwrap();
        } else {
            typed(&mut editor, "log");
        }
        assert!(editor.latex().is_err());
        let base = editor.cursor().row;
        typed(&mut editor, "n+1");
        assert_eq!(editor.cursor().row, base);
        editor.apply(Action::NextField(false)).unwrap();
        let argument = editor.cursor().row;
        assert_ne!(argument, base);
        typed(&mut editor, "x");
        assert_eq!(editor.latex().unwrap(), r"\log_{n+1}\left(x\right)");
        editor.apply(Action::NextField(true)).unwrap();
        assert_eq!(editor.cursor().row, base);
    }
}

#[test]
fn a_powered_logarithm_returns_from_its_exponent_to_its_argument() {
    let mut editor = editor();
    editor.import_latex(r"\log^{2}\left(x\right)").unwrap();
    editor.apply(Action::Move(Direction::Left, false)).unwrap();
    editor.apply(Action::NextField(true)).unwrap();
    let mut exponent = editor.clone();
    typed(&mut exponent, "q");
    assert_eq!(exponent.latex().unwrap(), r"\log^{2q}\left(x\right)");
    editor.apply(Action::NextField(false)).unwrap();
    typed(&mut editor, "q");
    assert_eq!(editor.latex().unwrap(), r"\log^{2}\left(qx\right)");
}

#[test]
fn a_square_group_after_a_logarithm_keeps_its_own_fields_and_delimiters() {
    let mut editor = editor();
    editor.import_latex(r"\log_{2}\left.x\right]").unwrap();
    editor.apply(Action::Move(Direction::Left, false)).unwrap();
    typed(&mut editor, "y");
    assert_eq!(editor.latex_preview(), r"\log_{2}\left.xy\right]");
    assert!(editor.latex().is_err());
    editor.apply(Action::NextField(true)).unwrap();
    typed(&mut editor, "q");
    assert_eq!(editor.latex().unwrap(), r"\log_{2}q\left[xy\right]");
}

#[test]
fn separate_groups_exit_at_their_own_left_edge() {
    for (source, expected) in [
        (r"\log_{2}\left[x\right]", r"\log_{2}q\left[x\right]"),
        (r"\log_{2}+\left(x\right)", r"\log_{2}+q\left(x\right)"),
        (r"\sin^{2}\left(x\right)", r"\sin^{2}q\left(x\right)"),
        (
            r"\log_{2}\left(x\right)\left(y\right)",
            r"\log_{2}\left(x\right)q\left(y\right)",
        ),
    ] {
        let mut document = editor();
        document.import_latex(source).unwrap();
        document
            .apply(Action::Move(Direction::Left, false))
            .unwrap();
        document.apply(Action::NextField(true)).unwrap();
        typed(&mut document, "q");
        assert_eq!(document.latex().unwrap(), expected, "{source}");
    }
}

#[test]
fn explicitly_leaving_a_logarithm_stays_in_the_enclosing_numerator() {
    for (exit, expected) in [
        (
            Action::Insert(" ".into()),
            r"\frac{\log^{2}\left(x\right)q}{d}",
        ),
        (
            Action::Escape(Direction::Left),
            r"\frac{q\log^{2}\left(x\right)}{d}",
        ),
    ] {
        let mut editor = editor();
        editor
            .import_latex(r"\frac{\log^{2}\left(x\right)}{d}")
            .unwrap();
        editor.apply(Action::RootHome(false)).unwrap();
        editor.apply(Action::Move(Direction::Right, false)).unwrap();
        editor.apply(Action::End(false)).unwrap();
        editor.apply(Action::Move(Direction::Left, false)).unwrap();
        if matches!(exit, Action::Insert(_)) {
            editor.apply(Action::NextField(true)).unwrap();
        }
        editor.apply(exit).unwrap();
        typed(&mut editor, "q");
        assert_eq!(editor.latex().unwrap(), expected);
    }
}

#[test]
fn bidirectional_log_fields_agree_after_typing_catalog_import_and_restore() {
    let source = r"\log_{b}^{2}\left(x\right)";
    let mut keyboard = editor();
    typed(&mut keyboard, "logb^2");
    keyboard.apply(Action::NextField(false)).unwrap();
    typed(&mut keyboard, "x");
    let mut catalog = editor();
    catalog.insert_catalog("log-base").unwrap();
    typed(&mut catalog, "b");
    catalog.apply(Action::Template(Template::Power)).unwrap();
    typed(&mut catalog, "2");
    catalog.apply(Action::NextField(false)).unwrap();
    typed(&mut catalog, "x");
    let mut imported = editor();
    imported.import_latex(source).unwrap();
    for (origin, mut document) in [
        ("keyboard", keyboard),
        ("catalog", catalog),
        ("LaTeX", imported),
    ] {
        assert_eq!(document.latex().unwrap(), source, "{origin}");
        document.apply(Action::RootEnd(false)).unwrap();
        document
            .apply(Action::Move(Direction::Left, false))
            .unwrap();
        document.apply(Action::NextField(true)).unwrap();
        let mut restored = editor();
        restored.restore_draft(&document.draft().unwrap()).unwrap();
        for mut version in [document, restored] {
            version.apply(Action::NextField(true)).unwrap();
            typed(&mut version, "q");
            version.apply(Action::NextField(false)).unwrap();
            typed(&mut version, "r");
            version.apply(Action::NextField(false)).unwrap();
            typed(&mut version, "s");
            assert_eq!(
                version.latex().unwrap(),
                r"\log_{bq}^{r2}\left(sx\right)",
                "{origin}"
            );
        }
    }
}

#[test]
fn written_log_syntax_survives_automatic_expansion() {
    for (text, expected) in [
        ("log_2(x)", r"\log_{2}\left(x\right)"),
        ("log(x)", r"\log \left(x\right)"),
        ("log_(n+1)(x)", r"\log_{\left(n+1\right)}\left(x\right)"),
    ] {
        let mut editor = editor();
        typed(&mut editor, text);
        assert_eq!(
            editor
                .latex()
                .unwrap_or_else(|error| panic!("{text}: {error}")),
            expected,
            "{text}"
        );
    }
}

#[test]
fn root_parentheses_close_the_root_before_the_next_operator() {
    for (text, expected) in [
        ("sqrt(3)/2", r"\frac{\sqrt{3}}{2}"),
        ("sqrt(3)+x", r"\sqrt{3}+x"),
        ("sqrt(sqrt(3))/2", r"\frac{\sqrt{\sqrt{3}}}{2}"),
        ("sqrt((a+b))/2", r"\frac{\sqrt{\left(a+b\right)}}{2}"),
    ] {
        let mut editor = editor();
        typed(&mut editor, text);
        assert_eq!(editor.latex().unwrap(), expected, "{text}");
    }
}

#[test]
fn tab_visits_fields_and_space_exits_without_moving_through_their_text() {
    let mut editor = editor();
    editor.apply(Action::Template(Template::Fraction)).unwrap();
    let numerator = editor.cursor().row;
    typed(&mut editor, "a+b");
    editor.apply(Action::NextField(false)).unwrap();
    let denominator = editor.cursor().row;
    assert_ne!(numerator, denominator);
    typed(&mut editor, "23");
    editor.apply(Action::NextField(true)).unwrap();
    assert_eq!(editor.cursor().row, numerator);
    editor.apply(Action::NextField(false)).unwrap();
    editor.apply(Action::Home(false)).unwrap();
    typed(&mut editor, " ");
    assert!(editor.at_root());
    typed(&mut editor, "+x");
    assert_eq!(editor.latex().unwrap(), r"\frac{a+b}{23}+x");
}

#[test]
fn undo_restores_literal_command_then_redo_restores_its_template() {
    for word in ["log", "sqrt", "sum", "lim", "sin"] {
        let mut editor = editor();
        typed(&mut editor, word);
        let expanded = editor.latex_preview();
        editor.apply(Action::Undo).unwrap();
        assert_eq!(editor.latex_preview(), word, "{word}");
        editor.apply(Action::Redo).unwrap();
        assert_eq!(editor.latex_preview(), expanded, "{word}");
    }
}

#[test]
fn exponent_scope_and_longer_function_names_remain_stable() {
    let mut editor = editor();
    typed(&mut editor, "P^n+m");
    assert_eq!(editor.latex().unwrap(), "P^{n+m}");
    editor.apply(Action::Escape(Direction::Right)).unwrap();
    typed(&mut editor, "+q");
    assert_eq!(editor.latex().unwrap(), "P^{n+m}+q");
    let mut editor = self::editor();
    typed(&mut editor, "sinh(x)");
    assert_eq!(editor.latex().unwrap(), r"\sinh \left(x\right)");
}

#[test]
fn a_draft_keeps_template_fields_and_the_hosts_typing_policy() {
    let mut editor = editor();
    typed(&mut editor, "log");
    let draft = editor.draft().unwrap();
    editor.restore_draft(&draft).unwrap();
    assert!(editor.structured_typing);
    typed(&mut editor, "2");
    editor.apply(Action::NextField(false)).unwrap();
    typed(&mut editor, "x");
    assert_eq!(editor.latex().unwrap(), r"\log_{2}\left(x\right)");
}

#[test]
fn space_separates_shortcuts_and_slot_paste_keeps_auto_parentheses_complete() {
    let mut editor = editor();
    typed(&mut editor, "p i");
    assert_eq!(editor.latex().unwrap(), "pi");
    let mut editor = self::editor();
    typed(&mut editor, "log2");
    editor.apply(Action::NextField(false)).unwrap();
    editor.paste(r"\frac{1}{2}").unwrap();
    assert_eq!(editor.latex().unwrap(), r"\log_{2}\left(\frac{1}{2}\right)");
}

#[test]
fn editing_inside_a_logarithm_base_keeps_parentheses_at_the_caret() {
    let mut editor = editor();
    typed(&mut editor, "log12");
    let base = editor.cursor().row;
    editor.apply(Action::Home(false)).unwrap();
    typed(&mut editor, "(3)");
    assert_eq!(editor.cursor().row, base);
    editor.apply(Action::NextField(false)).unwrap();
    typed(&mut editor, "x");
    assert_eq!(
        editor.latex().unwrap(),
        r"\log_{\left(3\right)12}\left(x\right)"
    );
}

#[test]
fn leaving_a_function_argument_confirms_its_suggested_closing_parenthesis() {
    for field_key in [false, true] {
        for (source, expected) in [
            ("sin(x", r"\sin \left(x\right)+q"),
            ("sqrt(3", r"\sqrt{3}+q"),
        ] {
            let mut editor = editor();
            typed(&mut editor, source);
            assert!(editor.latex().is_err(), "unfinished: {source}");
            if field_key {
                editor.apply(Action::NextField(false)).unwrap();
            } else {
                typed(&mut editor, " ");
            }
            typed(&mut editor, "+q");
            assert_eq!(
                editor.latex().unwrap(),
                expected,
                "{source}, field={field_key}"
            );
        }
    }
}

#[cfg(feature = "slint-ui")]
#[test]
fn physical_and_onscreen_field_navigation_use_the_same_typing_policy() {
    use slint::platform::Key;
    use slint_math_input::gui::{ClipboardAccess, MathInputController};
    struct NoClipboard;
    impl ClipboardAccess for NoClipboard {
        fn read_text(&mut self) -> Result<String, slint_math_input::Error> {
            unreachable!("no clipboard read");
        }
        fn write_text(&mut self, _: String) -> Result<(), slint_math_input::Error> {
            unreachable!("no clipboard write");
        }
    }
    for onscreen in [false, true] {
        let mut input = MathInputController::with_clipboard(NoClipboard);
        input.editor.structured_typing = true;
        input.keypad("catalog:fraction", false);
        input.insert("a+b");
        let numerator = input.editor.cursor().row;
        if onscreen {
            input.keypad("key:next", false);
        } else {
            assert!(input.key(&char::from(Key::Tab).to_string(), false, false, false));
        }
        input.insert("2");
        assert_eq!(input.editor.latex().unwrap(), r"\frac{a+b}{2}");
        assert!(input.key(&char::from(Key::Tab).to_string(), false, true, false));
        assert_eq!(input.editor.cursor().row, numerator);
        input.insert(" ");
        assert!(input.editor.at_root());
        assert!(input.error.is_none());
    }
}
