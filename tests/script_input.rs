//! Keyboard and pointer edits preserve script scope and structural boundaries.
use slint_math_input::{Action, Direction, Editor, Position, Template};

fn type_text(editor: &mut Editor, text: &str) {
    for ch in text.chars() {
        editor.apply(Action::Insert(ch.to_string())).unwrap();
    }
}

fn editor_from_latex(source: &str, font_size: f32) -> Editor {
    let mut editor = Editor::new();
    editor.font_size = font_size;
    editor.import_latex(source).unwrap();
    editor
}

fn restore_editor(draft: &str, font_size: f32) -> Editor {
    let mut editor = Editor::from_draft(draft).unwrap();
    // Font size is presentation state and is deliberately absent from drafts.
    editor.font_size = font_size;
    editor
}

fn click_near_caret(editor: &mut Editor, target: Position, dx: f32, height_fraction: f32) {
    let caret = editor.layout().caret(target).unwrap().rect;
    editor
        .pointer(
            caret.x + dx,
            caret.y + caret.height * height_fraction,
            false,
        )
        .unwrap();
}

// Pointer placement and structural scope.

#[test]
fn clicking_before_scripts_keeps_coefficients_outside_the_base() {
    for source in [
        r"\log_{b}",
        r"\log_{bx}",
        r"\log_{b}\left(x\right)",
        "x^{2}",
        "x_{n}^{2}",
        "{x^{2}}^{3}",
        r"\mathbf{x}_{n}",
        r"\mathbf{\mathit{x}}_{n}",
        r"\overline{x}_{n}",
        r"\overset{a}{x}_{n}",
    ] {
        for size in [18.0, 32.0, 60.0] {
            let seed = editor_from_latex(source, size);
            let start = Position {
                row: seed.cursor().row,
                offset: 0,
            };
            let draft = seed.draft().unwrap();
            let expected = format!("y{}", seed.latex().unwrap());
            let mut control = restore_editor(&draft, size);
            control.set_position(start, false).unwrap();
            type_text(&mut control, "y");
            assert_eq!(control.latex().unwrap(), expected, "explicit root position");
            for fraction in [0.25, 0.5, 0.75] {
                for dx in [-size * 0.03, 0.0, size * 0.03] {
                    let mut editor = restore_editor(&draft, size);
                    click_near_caret(&mut editor, start, dx, fraction);
                    let hit = editor.cursor();
                    type_text(&mut editor, "y");
                    assert_eq!(
                        editor.latex().unwrap(),
                        expected,
                        "{source}, size {size}, dx {dx}, height {fraction}: hit {hit:?}, intended {start:?}"
                    );
                    assert_eq!(hit, start);
                    editor.apply(Action::Undo).unwrap();
                    assert_eq!(editor.latex().unwrap(), source);
                    editor.apply(Action::Redo).unwrap();
                    assert_eq!(editor.latex().unwrap(), expected);
                    assert_eq!(
                        Editor::from_draft(&editor.draft().unwrap())
                            .unwrap()
                            .latex()
                            .unwrap(),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn clicking_before_a_script_targets_its_immediate_parent_row() {
    for (source, nested, offset, expected) in [
        (
            r"a+\log_{b}\left(x\right)",
            false,
            2,
            r"a+y\log_{b}\left(x\right)",
        ),
        (
            r"\frac{\log_{b}\left(x\right)}{2}",
            true,
            0,
            r"\frac{y\log_{b}\left(x\right)}{2}",
        ),
    ] {
        let mut editor = editor_from_latex(source, 32.0);
        editor.apply(Action::RootHome(false)).unwrap();
        if nested {
            editor.apply(Action::Move(Direction::Right, false)).unwrap();
        }
        let start = Position {
            row: editor.cursor().row,
            offset,
        };
        click_near_caret(&mut editor, start, 1.0, 0.25);
        assert_eq!(editor.cursor(), start);
        type_text(&mut editor, "y");
        assert_eq!(editor.latex().unwrap(), expected);
    }
}

#[test]
fn grouped_script_bases_remain_editable_by_pointer() {
    for (source, steps, expected) in [
        ("{ab}^{2}", 1, "{yab}^{2}"),
        (r"\mathbf{ab}_{n}", 2, r"\mathbf{yab}_{n}"),
    ] {
        let mut editor = editor_from_latex(source, 32.0);
        editor.apply(Action::RootHome(false)).unwrap();
        let root = editor.cursor();
        for _ in 0..steps {
            editor.apply(Action::Move(Direction::Right, false)).unwrap();
        }
        let inside = editor.cursor();
        assert_ne!(inside.row, root.row);
        click_near_caret(&mut editor, inside, 1.0, 0.5);
        assert_eq!(editor.cursor(), inside);
        type_text(&mut editor, "y");
        assert_eq!(editor.latex().unwrap(), expected);
    }
}

#[test]
fn clicking_outside_the_formula_keeps_text_out_of_nested_slots() {
    for source in [
        r"\sqrt[n]{x}",
        r"\sqrt[\frac{1}{n}]{x}",
        r"\frac{a}{b}",
        r"\sum_{n=1}^{10}x",
    ] {
        for size in [18.0, 32.0, 60.0] {
            let seed = editor_from_latex(source, size);
            let end = seed.cursor();
            let draft = seed.draft().unwrap();
            for target in [
                Position {
                    row: end.row,
                    offset: 0,
                },
                end,
            ] {
                let mut control = restore_editor(&draft, size);
                control.set_position(target, false).unwrap();
                type_text(&mut control, "y");
                let expected = control.latex().unwrap();
                let sign = if target.offset == 0 { -1.0 } else { 1.0 };
                for fraction in [0.25, 0.5, 0.75] {
                    let mut editor = restore_editor(&draft, size);
                    click_near_caret(&mut editor, target, sign * size * 0.03, fraction);
                    let hit = editor.cursor();
                    type_text(&mut editor, "y");
                    assert_eq!(
                        editor.latex().unwrap(),
                        expected,
                        "{source}, size {size}, height {fraction}, hit {hit:?}, target {target:?}"
                    );
                    assert_eq!(hit, target);
                }
            }
        }
    }
}

#[test]
fn empty_script_bases_remain_fillable_by_pointer() {
    let mut editor = editor_from_latex("{}_{b}", 32.0);
    editor.apply(Action::RootHome(false)).unwrap();
    editor.apply(Action::Move(Direction::Right, false)).unwrap();
    let inside = editor.cursor();
    click_near_caret(&mut editor, inside, 0.0, 0.5);
    assert_eq!(editor.cursor(), inside);
    type_text(&mut editor, "x");
    assert_eq!(editor.latex().unwrap(), "x_{b}");
}

#[test]
fn pointer_reentry_and_restored_drafts_keep_operators_in_the_script() {
    let mut editor = editor_from_latex("P^{n}", 32.0);
    editor.apply(Action::Move(Direction::Left, false)).unwrap();
    let inside = editor.cursor();
    editor.apply(Action::Escape(Direction::Right)).unwrap();
    assert!(editor.at_root());
    click_near_caret(&mut editor, inside, 0.0, 0.5);
    assert_eq!(editor.cursor(), inside);
    let mut restored = restore_editor(&editor.draft().unwrap(), editor.font_size);
    type_text(&mut restored, "+m");
    assert_eq!(restored.latex().unwrap(), "P^{n+m}");
}

// Keyboard input and history.

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
fn opposite_script_templates_use_the_same_base_and_preserve_history() {
    for (prefix, template, suffix, expected) in [
        ("x_1", Template::Power, "2", "x_{1}^{2}"),
        ("x^2", Template::Subscript, "1", "x_{1}^{2}"),
        ("x_1^2", Template::Subscript, "3", "x_{13}^{2}"),
        ("x^2_1", Template::Power, "3", "x_{1}^{23}"),
    ] {
        let mut editor = Editor::new();
        type_text(&mut editor, prefix);
        let before = editor.draft().unwrap();
        editor.apply(Action::Template(template)).unwrap();
        let positioned = editor.draft().unwrap();
        type_text(&mut editor, suffix);
        assert_eq!(editor.latex().unwrap(), expected, "{prefix}");
        let after = editor.draft().unwrap();
        editor.apply(Action::Undo).unwrap();
        assert_eq!(editor.draft().unwrap(), positioned);
        if prefix == "x_1" || prefix == "x^2" {
            // Creating a missing script is an edit; revisiting one is navigation.
            editor.apply(Action::Undo).unwrap();
            assert_eq!(editor.draft().unwrap(), before);
            editor.apply(Action::Redo).unwrap();
        }
        editor.apply(Action::Redo).unwrap();
        assert_eq!(editor.draft().unwrap(), after);
    }
}

#[test]
fn script_templates_keep_mid_row_and_selected_edits_inside_the_active_script() {
    for selected in [false, true] {
        for template in [false, true] {
            let mut editor = Editor::new();
            type_text(&mut editor, "x_12");
            editor
                .apply(Action::Move(Direction::Left, selected))
                .unwrap();
            editor
                .apply(if template {
                    Action::Template(Template::Power)
                } else {
                    Action::Insert("^".into())
                })
                .unwrap();
            if !selected {
                type_text(&mut editor, "3");
            }
            assert_eq!(
                editor.latex().unwrap(),
                if selected { "x_{1^{2}}" } else { "x_{1^{3}2}" }
            );
        }
    }
}

// Slint controller input.

#[cfg(feature = "slint-ui")]
#[test]
fn mixed_input_can_prepend_a_coefficient_to_an_existing_logarithm() {
    use slint_math_input::gui::MathInputController;
    let mut controller = MathInputController::new();
    controller.keypad("catalog:log-base", false);
    controller.insert("b");
    controller.keypad("key:next", false);
    controller.insert("x)");
    let start = Position {
        row: controller.editor.cursor().row,
        offset: 0,
    };
    let caret = controller.editor.layout().caret(start).unwrap().rect;
    controller.pointer(caret.x + 1.0, caret.y + caret.height * 0.25, false);
    controller.insert("y");
    assert!(controller.error.is_none(), "{:?}", controller.error);
    assert_eq!(
        controller.editor.latex().unwrap(),
        r"y\log_{b}\left(x\right)"
    );
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

#[cfg(feature = "slint-ui")]
#[test]
fn keypad_and_keyboard_attach_opposite_scripts_to_the_same_base() {
    use slint_math_input::gui::MathInputController;
    for (prefix, token, suffix) in [("x_1", "key:power", "2"), ("x^2", "key:subscript", "1")] {
        let mut controller = MathInputController::new();
        controller.insert(prefix);
        controller.keypad(token, false);
        controller.insert(suffix);
        assert!(controller.error.is_none(), "{:?}", controller.error);
        assert_eq!(controller.editor.latex().unwrap(), "x_{1}^{2}");
    }
}

#[test]
fn clicking_after_a_scripted_function_targets_its_parent_baseline() {
    for source in [r"\log_{b}", "x^{2}", "x_{n}"] {
        for size in [18.0, 32.0, 60.0] {
            let seed = editor_from_latex(source, size);
            let end = seed.cursor();
            let draft = seed.draft().unwrap();
            for fraction in [0.25, 0.5, 0.75] {
                for dx in [-size * 0.03, 0.0, size * 0.03] {
                    let mut editor = restore_editor(&draft, size);
                    click_near_caret(&mut editor, end, dx, fraction);
                    let hit = editor.cursor();
                    type_text(&mut editor, "y");
                    assert_eq!(
                        editor.latex().unwrap(),
                        format!("{source}y"),
                        "{source}, size {size}, dx {dx}, height {fraction}: hit {hit:?}, intended {end:?}"
                    );
                }
            }
        }
    }
}

#[cfg(feature = "slint-ui")]
#[test]
fn base_logarithm_has_an_argument_after_leaving_its_base() {
    use slint_math_input::gui::MathInputController;
    let mut controller = MathInputController::new();
    controller.keypad("catalog:log-base", false);
    controller.insert("b");
    controller.keypad("key:next", false);
    controller.insert("x");
    assert!(controller.error.is_none(), "{:?}", controller.error);
    assert_eq!(
        controller.editor.latex().unwrap(),
        r"\log_{b}\left(x\right)"
    );
}
#[test]
fn logarithm_template_navigates_from_base_to_argument_and_back_to_baseline() {
    for leave in [
        Action::Move(Direction::Right, false),
        Action::Escape(Direction::Right),
    ] {
        for prefix in ["", "a+"] {
            let mut editor = Editor::new();
            type_text(&mut editor, prefix);
            editor.insert_catalog("log-base").unwrap();
            type_text(&mut editor, "a+1");
            let base = editor.cursor();
            let base_height = editor.layout().caret(base).unwrap().rect.height;
            assert!(editor.latex().is_err(), "the argument is still empty");
            editor.apply(leave.clone()).unwrap();
            let argument = editor.cursor();
            assert_ne!(argument.row, base.row);
            assert!(editor.layout().caret(argument).unwrap().rect.height > base_height);
            type_text(&mut editor, "x");
            let expected = format!(r"{prefix}\log_{{a+1}}\left(x\right)");
            assert_eq!(editor.latex().unwrap(), expected);
            let draft = editor.draft().unwrap();
            editor.apply(Action::Undo).unwrap();
            assert!(editor.latex().is_err());
            editor.apply(Action::Redo).unwrap();
            assert_eq!(editor.draft().unwrap(), draft);
            let mut editor = Editor::from_draft(&draft).unwrap();
            type_text(&mut editor, ")");
            type_text(&mut editor, "-");
            editor.insert_catalog("log-base").unwrap();
            type_text(&mut editor, "b");
            editor.apply(leave.clone()).unwrap();
            type_text(&mut editor, "y");
            assert_eq!(
                editor.latex().unwrap(),
                format!(r"{expected}-\log_{{b}}\left(y\right)")
            );
            editor.apply(leave.clone()).unwrap();
            type_text(&mut editor, "+z");
            assert_eq!(
                editor.latex().unwrap(),
                format!(r"{expected}-\log_{{b}}\left(y\right)+z")
            );
        }
    }
}

#[test]
fn logarithm_template_wraps_the_selection_in_its_argument() {
    let mut editor = Editor::new();
    type_text(&mut editor, "x+y");
    editor.apply(Action::SelectAll).unwrap();
    editor.insert_catalog("log-base").unwrap();
    type_text(&mut editor, "b");
    assert_eq!(editor.latex().unwrap(), r"\log_{b}\left(x+y\right)");
    editor.apply(Action::Escape(Direction::Right)).unwrap();
    assert!(!editor.at_root());
    editor.apply(Action::End(false)).unwrap();
    type_text(&mut editor, "+z");
    assert_eq!(editor.latex().unwrap(), r"\log_{b}\left(x+y+z\right)");
}

#[test]
fn powered_logarithms_advance_from_either_script_into_the_argument() {
    let mut seed = Editor::new();
    seed.insert_catalog("log-base").unwrap();
    type_text(&mut seed, "b");
    let base = seed.cursor();
    type_text(&mut seed, "^2");
    let exponent = seed.cursor();
    let draft = seed.draft().unwrap();
    for position in [base, exponent] {
        for leave in [
            Action::Move(Direction::Right, false),
            Action::Escape(Direction::Right),
        ] {
            let mut editor = Editor::from_draft(&draft).unwrap();
            editor.set_position(position, false).unwrap();
            editor.apply(leave.clone()).unwrap();
            assert!(!editor.at_root());
            type_text(&mut editor, "x");
            assert_eq!(editor.latex().unwrap(), r"\log_{b}^{2}\left(x\right)");
            let filled = editor.draft().unwrap();
            editor.apply(Action::Undo).unwrap();
            assert!(editor.latex().is_err());
            editor.apply(Action::Redo).unwrap();
            assert_eq!(editor.draft().unwrap(), filled);
            editor.apply(leave).unwrap();
            type_text(&mut editor, "+y");
            assert_eq!(editor.latex().unwrap(), r"\log_{b}^{2}\left(x\right)+y");
        }
    }
}

#[cfg(feature = "slint-ui")]
#[test]
fn powered_logarithms_accept_controller_navigation_from_keyboard_and_keypad() {
    use slint_math_input::gui::MathInputController;
    for (key, token) in [
        (slint::platform::Key::RightArrow, "key:right"),
        (slint::platform::Key::Tab, "key:next"),
        (slint::platform::Key::Escape, "key:next"),
    ] {
        for keypad in [false, true] {
            let mut controller = MathInputController::new();
            controller.keypad("catalog:log-base", false);
            controller.insert("b^2");
            if keypad {
                controller.keypad(token, false);
            } else {
                assert!(controller.key(&char::from(key).to_string(), false, false, false));
            }
            controller.insert("x");
            assert!(controller.error.is_none(), "{:?}", controller.error);
            assert_eq!(
                controller.editor.latex().unwrap(),
                r"\log_{b}^{2}\left(x\right)"
            );
        }
    }
}

#[test]
fn bare_logarithms_and_other_scripts_keep_their_existing_exit_behavior() {
    for source in [
        r"\log_{b}",
        r"\log_{b}^{2}",
        r"x_{b}\left(y\right)",
        r"x_{b}^{2}\left(y\right)",
    ] {
        let mut editor = editor_from_latex(source, 32.0);
        editor.apply(Action::RootHome(false)).unwrap();
        editor.apply(Action::Move(Direction::Right, false)).unwrap();
        editor.apply(Action::Move(Direction::Right, false)).unwrap();
        editor.apply(Action::Move(Direction::Right, false)).unwrap();
        editor.apply(Action::End(false)).unwrap();
        editor.apply(Action::Escape(Direction::Right)).unwrap();
        assert!(editor.at_root(), "{source}");
        assert_eq!(editor.cursor().offset, 1);
    }
}
