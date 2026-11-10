//! Fixed editing tasks across core, controller, history and draft checkpoints.
use slint_math_input::{Action, Direction, Editor, MatrixStyle, Template};

struct Workflow {
    name: &'static str,
    actions: Vec<Action>,
    expected: &'static str,
}

fn text(value: &str) -> Action {
    Action::Insert(value.into())
}
fn right() -> Action {
    Action::Move(Direction::Right, false)
}
fn template(value: Template) -> Action {
    Action::Template(value)
}

fn workflows() -> Vec<Workflow> {
    vec![
        Workflow {
            name: "fraction followed by a baseline term",
            actions: vec![text("a+b/2"), right(), text("+y")],
            expected: r"a+\frac{b}{2}+y",
        },
        Workflow {
            name: "fraction inside a square root",
            actions: vec![
                template(Template::Root),
                text("x/2"),
                right(),
                right(),
                text("+y"),
            ],
            expected: r"\sqrt{\frac{x}{2}}+y",
        },
        Workflow {
            name: "fraction inside an exponent",
            actions: vec![text("x^1/2"), right(), right(), text("+y")],
            expected: r"x^{\frac{1}{2}}+y",
        },
        Workflow {
            name: "indexed root with a fractional body",
            actions: vec![
                template(Template::IndexedRoot),
                text("12"),
                right(),
                text("x/2"),
                right(),
                right(),
                text("+1"),
            ],
            expected: r"\sqrt[12]{\frac{x}{2}}+1",
        },
        Workflow {
            name: "logarithm with a fractional argument",
            actions: vec![
                template(Template::LogBase),
                text("b"),
                right(),
                text("x/2"),
                right(),
                right(),
                text("+1"),
            ],
            expected: r"\log_{b}\left(\frac{x}{2}\right)+1",
        },
        Workflow {
            name: "powered logarithm",
            actions: vec![
                template(Template::LogBase),
                text("b"),
                template(Template::Power),
                text("2"),
                right(),
                text("x"),
                right(),
                text("+y"),
            ],
            expected: r"\log_{b}^{2}\left(x\right)+y",
        },
        Workflow {
            name: "combined scripts",
            actions: vec![
                text("x"),
                template(Template::Subscript),
                text("12"),
                template(Template::Power),
                text("2"),
                right(),
                text("+y"),
            ],
            expected: r"x_{12}^{2}+y",
        },
        Workflow {
            name: "named functions and paired arguments",
            actions: vec![text("sin(x)+cos(y)")],
            expected: r"\sin \left(x\right)+\cos \left(y\right)",
        },
        Workflow {
            name: "limit followed by a fraction",
            actions: vec![
                template(Template::Limit("lim".into())),
                text("x->0"),
                right(),
                text("sin(x)/x"),
            ],
            expected: r"\lim_{x\to 0}\frac{\sin \left(x\right)}{x}",
        },
        Workflow {
            name: "sum with bounds and a scripted term",
            actions: vec![
                template(Template::Sum),
                text("1"),
                right(),
                text("10"),
                right(),
                text("x^2"),
                right(),
                text("+y"),
            ],
            expected: r"\sum_{n=1}^{10} x^{2}+y",
        },
        Workflow {
            name: "product with bounds and a subscripted term",
            actions: vec![
                template(Template::Product),
                text("1"),
                right(),
                text("10"),
                right(),
                text("x_n"),
                right(),
                text("+y"),
            ],
            expected: r"\prod_{n=1}^{10} x_{n}+y",
        },
        Workflow {
            name: "definite integral",
            actions: vec![
                template(Template::Integral),
                text("0"),
                right(),
                text("1"),
                right(),
                text("f(x)dx"),
            ],
            expected: r"\int_{0}^{1} f\left(x\right)dx",
        },
        Workflow {
            name: "binomial coefficient",
            actions: vec![
                template(Template::Binomial),
                text("n"),
                right(),
                text("k"),
                right(),
                text("+1"),
            ],
            expected: r"\binom{n}{k}+1",
        },
        Workflow {
            name: "two by two matrix",
            actions: vec![
                template(Template::Matrix {
                    rows: 2,
                    columns: 2,
                }),
                text("12"),
                right(),
                text("34"),
                right(),
                text("56"),
                right(),
                text("78"),
                right(),
                text("+A"),
            ],
            expected: r"\begin{bmatrix}12&34\\56&78\end{bmatrix}+A",
        },
        Workflow {
            name: "piecewise expression",
            actions: vec![
                template(Template::StyledMatrix {
                    rows: 2,
                    columns: 2,
                    style: MatrixStyle::Cases,
                }),
                text("x"),
                right(),
                text("x>=0"),
                right(),
                text("-x"),
                right(),
                text("x<0"),
                right(),
            ],
            expected: r"\begin{cases}x&x\ge 0\\-x&x<0\end{cases}",
        },
        Workflow {
            name: "wrap a selection in a root",
            actions: vec![
                text("x+y"),
                Action::SelectAll,
                template(Template::Root),
                right(),
                text("+z"),
            ],
            expected: r"\sqrt{x+y}+z",
        },
        Workflow {
            name: "wrap a selection in a logarithm",
            actions: vec![
                text("x+y"),
                Action::SelectAll,
                template(Template::LogBase),
                text("b"),
                right(),
                Action::End(false),
                right(),
                text("+z"),
            ],
            expected: r"\log_{b}\left(x+y\right)+z",
        },
        Workflow {
            name: "derivative followed by its expression",
            actions: vec![
                template(Template::Derivative {
                    partial: false,
                    order: 1,
                }),
                text("sin(x)"),
            ],
            expected: r"\frac{d}{dx}\sin \left(x\right)",
        },
    ]
}

fn finish(editor: &mut Editor, actions: &[Action]) {
    for action in actions {
        editor.apply(action.clone()).unwrap();
        editor.validate().unwrap();
    }
}

#[test]
fn common_editing_workflows_have_fixed_results() {
    let mut failures = vec![];
    for workflow in workflows() {
        let mut editor = Editor::new();
        finish(&mut editor, &workflow.actions);
        if editor.latex().as_deref() != Ok(workflow.expected) {
            failures.push(format!(
                "{}: {} != {}",
                workflow.name,
                editor.latex_preview(),
                workflow.expected
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn workflows_resume_from_every_draft_checkpoint() {
    for workflow in workflows() {
        let mut expected = Editor::new();
        finish(&mut expected, &workflow.actions);
        let mut editor = Editor::new();
        for checkpoint in 0..=workflow.actions.len() {
            let draft = editor.draft().unwrap();
            let mut restored = Editor::from_draft(&draft).unwrap();
            assert_eq!(restored.draft().unwrap(), draft);
            finish(&mut restored, &workflow.actions[checkpoint..]);
            assert_eq!(
                restored.draft().unwrap(),
                expected.draft().unwrap(),
                "{}, checkpoint {checkpoint}",
                workflow.name
            );
            if let Some(action) = workflow.actions.get(checkpoint) {
                editor.apply(action.clone()).unwrap();
            }
        }
    }
}

#[test]
fn workflow_edits_restore_content_cursor_and_selection_through_history() {
    for workflow in workflows() {
        let mut editor = Editor::new();
        for action in workflow.actions {
            let before = editor.draft().unwrap();
            let preview = editor.latex_preview();
            editor.apply(action.clone()).unwrap();
            if editor.latex_preview() != preview {
                let after = editor.draft().unwrap();
                editor.apply(Action::Undo).unwrap();
                assert_eq!(
                    editor.draft().unwrap(),
                    before,
                    "{}, undo {action:?}",
                    workflow.name
                );
                editor.apply(Action::Redo).unwrap();
                assert_eq!(
                    editor.draft().unwrap(),
                    after,
                    "{}, redo {action:?}",
                    workflow.name
                );
            }
        }
    }
}

#[test]
fn copying_and_pasting_workflows_keeps_the_host_scope_and_history() {
    for workflow in workflows() {
        let mut source = Editor::new();
        finish(&mut source, &workflow.actions);
        let copied = source.copy_latex();
        for nested in [false, true] {
            let mut host = Editor::new();
            host.apply(text(if nested { "z^" } else { "a+" })).unwrap();
            let before = host.draft().unwrap();
            host.paste(&copied).unwrap();
            let pasted = host.draft().unwrap();
            host.apply(Action::Undo).unwrap();
            assert_eq!(host.draft().unwrap(), before);
            host.apply(Action::Redo).unwrap();
            assert_eq!(host.draft().unwrap(), pasted);
            if nested {
                host.apply(Action::Escape(Direction::Right)).unwrap();
            }
            host.apply(text("+b")).unwrap();
            let expected = if nested {
                format!("z^{{{copied}}}+b")
            } else {
                format!("a+{copied}+b")
            };
            assert_eq!(
                host.latex().unwrap(),
                expected,
                "{}, nested={nested}",
                workflow.name
            );
        }
    }
}

#[test]
fn workflows_keep_their_scope_inside_an_enclosing_structure() {
    for workflow in workflows() {
        // SelectAll deliberately selects the whole document, including the host.
        if workflow.actions.contains(&Action::SelectAll) {
            continue;
        }
        for (source, open, close) in [
            (r"\sqrt{}", r"\sqrt{", "}"),
            (r"\left(\right)", r"\left(", r"\right)"),
        ] {
            let mut editor = Editor::new();
            editor.import_latex(source).unwrap();
            editor.apply(Action::Move(Direction::Left, false)).unwrap();
            finish(&mut editor, &workflow.actions);
            assert_eq!(
                editor.latex().unwrap(),
                format!("{open}{}{close}", workflow.expected),
                "{}, in {source}",
                workflow.name
            );
        }
    }
}

#[test]
fn boundary_deletion_preserves_all_children_and_surrounding_terms() {
    let mut failures = vec![];
    for (source, contents) in [
        (r"\frac{12}{34}", "1234"),
        (r"\sqrt[3]{12}", "312"),
        (r"\binom{12}{34}", "1234"),
        (r"\begin{bmatrix}12&34\\56&78\end{bmatrix}", "12345678"),
        (r"\sum_{n=1}^{N}", "n=1N"),
        (r"x_{12}^{34}", "x1234"),
    ] {
        for forward in [false, true] {
            let mut editor = Editor::new();
            editor.import_latex(&format!("a+{source}+b")).unwrap();
            editor
                .apply(if forward {
                    Action::RootEnd(false)
                } else {
                    Action::RootHome(false)
                })
                .unwrap();
            for _ in 0..3 {
                editor
                    .apply(Action::Move(
                        if forward {
                            Direction::Left
                        } else {
                            Direction::Right
                        },
                        false,
                    ))
                    .unwrap();
            }
            editor
                .apply(if forward {
                    Action::End(false)
                } else {
                    Action::Home(false)
                })
                .unwrap();
            let before = editor.draft().unwrap();
            editor
                .apply(if forward {
                    Action::Delete
                } else {
                    Action::Backspace
                })
                .unwrap();
            let expected = format!("a+{contents}+b");
            if editor.latex().as_deref() != Ok(expected.as_str()) {
                failures.push(format!(
                    "{source}, forward={forward}: {} != {expected}",
                    editor.latex_preview()
                ));
            }
            editor.apply(Action::Undo).unwrap();
            assert_eq!(editor.draft().unwrap(), before);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn pointer_reentry_between_atoms_matches_keyboard_insertion() {
    let mut failures = vec![];
    for source in [
        r"a+\frac{12}{34}+b",
        r"a+\sqrt[12]{34}+b",
        r"a+x_{12}^{34}+b",
        r"a+\log_{12}^{34}\left(56\right)+b",
        r"a+\sum_{n=12}^{34}x+b",
        r"a+\int_{12}^{34}x+b",
        r"a+\lim_{x\to12}f(x)+b",
        r"a+\begin{bmatrix}12&34\\56&78\end{bmatrix}+b",
        r"a+\begin{cases}12&x>0\\34&x<0\end{cases}+b",
        r"a+\widehat{xy}+b",
        r"a+\mathbf{xy}+b",
        r"a+\overset{ab}{xy}+b",
        r"a+\frac{x_{12}}{\sqrt[34]{56}}+b",
        r"a+\left(12+\frac{34}{56}\right)+b",
    ] {
        for size in [18.0, 32.0, 60.0] {
            let mut seed = Editor::new();
            seed.import_latex(source).unwrap();
            seed.font_size = size;
            let root = seed.cursor().row;
            let layout = seed.layout();
            let mut checked_nested = false;
            for caret in &layout.carets {
                let end = layout
                    .carets
                    .iter()
                    .filter(|candidate| candidate.position.row == caret.position.row)
                    .map(|candidate| candidate.position.offset)
                    .max()
                    .unwrap();
                if caret.position.offset == 0 || caret.position.offset == end {
                    continue;
                }
                let center_y = caret.rect.y + caret.rect.height * 0.5;
                // Shared visual boundaries have an explicit ownership policy;
                // this check covers unambiguous positions between visible atoms.
                if layout.carets.iter().any(|other| {
                    other.position.row != caret.position.row
                        && (other.rect.x - caret.rect.x).abs() < size * 0.05
                        && (other.rect.y + other.rect.height * 0.5 - center_y).abs() < size * 0.05
                }) {
                    continue;
                }
                checked_nested |= caret.position.row != root;
                let mut expected = seed.clone();
                expected.set_position(caret.position, false).unwrap();
                expected.apply(text("z")).unwrap();
                let mut actual = seed.clone();
                actual.pointer(caret.rect.x, center_y, false).unwrap();
                actual.apply(text("z")).unwrap();
                if actual.latex().unwrap() != expected.latex().unwrap() {
                    failures.push(format!(
                        "{source}, size={size}, intended {:?}: {} != {}",
                        caret.position,
                        actual.latex_preview(),
                        expected.latex_preview()
                    ));
                }
            }
            assert!(
                checked_nested,
                "{source}, size={size}: no nested insertion point checked"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[cfg(feature = "slint-ui")]
fn controller_action(
    controller: &mut slint_math_input::gui::MathInputController,
    action: &Action,
    keypad: bool,
) {
    use slint::platform::Key;
    match action {
        Action::Insert(value) => {
            if keypad {
                for ch in value.chars() {
                    controller.keypad(&ch.to_string(), false);
                    assert!(controller.error.is_none());
                }
            } else {
                controller.insert(value);
            }
        }
        Action::Template(template) => {
            let token = match template {
                Template::Power => "key:power".into(),
                Template::Subscript => "key:subscript".into(),
                Template::Root => "key:root".into(),
                Template::IndexedRoot => "key:indexed-root".into(),
                Template::Matrix {
                    rows: 2,
                    columns: 2,
                } => "key:matrix".into(),
                _ => format!(
                    "catalog:{}",
                    slint_math_input::catalog::entries()
                        .iter()
                        .find(|entry| entry.action == *action)
                        .expect("workflow template has a catalog entry")
                        .id
                ),
            };
            controller.keypad(&token, false);
        }
        Action::Move(direction, extend) => {
            let (key, token) = match direction {
                Direction::Left => (Key::LeftArrow, "key:left"),
                Direction::Right => (Key::RightArrow, "key:right"),
                Direction::Up => (Key::UpArrow, "key:up"),
                Direction::Down => (Key::DownArrow, "key:down"),
            };
            if keypad {
                controller.keypad(token, *extend);
            } else {
                assert!(controller.key(&char::from(key).to_string(), false, *extend, false));
            }
        }
        Action::End(extend) => {
            assert!(controller.key(&char::from(Key::End).to_string(), false, *extend, false));
        }
        Action::SelectAll => {
            assert!(controller.key("a", true, false, false));
        }
        other => panic!("unmapped workflow action: {other:?}"),
    }
    assert!(
        controller.error.is_none(),
        "{action:?}: {:?}",
        controller.error
    );
}

#[cfg(feature = "slint-ui")]
#[test]
fn keyboard_and_mouse_keypad_workflows_match_the_core_at_every_step() {
    use slint_math_input::gui::MathInputController;
    for workflow in workflows() {
        for keypad in [false, true] {
            let mut core = Editor::new();
            let mut controller = MathInputController::new();
            for action in &workflow.actions {
                core.apply(action.clone()).unwrap();
                controller_action(&mut controller, action, keypad);
                assert_eq!(
                    controller.editor.draft().unwrap(),
                    core.draft().unwrap(),
                    "{}, {action:?}, keypad={keypad}",
                    workflow.name
                );
            }
            assert_eq!(
                controller.editor.latex().unwrap(),
                workflow.expected,
                "{}",
                workflow.name
            );
        }
    }
}
