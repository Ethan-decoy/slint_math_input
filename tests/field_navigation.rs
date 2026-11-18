//! Check field navigation against marked-up LaTeX, independently of the editor tree.
use slint_math_input::{Action, Direction, Editor};
use std::collections::VecDeque;

#[derive(Clone)]
struct Expected {
    slots: Vec<String>,
    field: usize,
    offset: usize,
}

impl Expected {
    fn new(slots: &[&str]) -> Self {
        // The outer slots hold text before and after the structure. Left from
        // the end of each fixture enters the end of its last editable field.
        let field = slots.len() - 2;
        Self {
            slots: slots.iter().map(|s| s.to_string()).collect(),
            field,
            offset: slots[field].len(),
        }
    }

    fn formula(&self, pattern: &str, marker: &str) -> String {
        let mut slots = self.slots.clone();
        slots[self.field].insert_str(self.offset, marker);
        let mut pieces = pattern.split('|');
        let mut output = pieces.next().unwrap().to_owned();
        for slot in slots {
            output.push_str(&slot);
            output.push_str(pieces.next().unwrap());
        }
        assert!(pieces.next().is_none());
        output
    }

    fn tab(&mut self, backward: bool) {
        if self.field == 0 || self.field == self.slots.len() - 1 {
            return;
        }
        if backward {
            self.field -= 1;
            self.offset = self.slots[self.field].len();
        } else {
            self.field += 1;
            self.offset = 0;
        }
    }
}

#[test]
fn field_edits_and_history_follow_the_declared_slots() {
    // Each | is a distinct insertion slot, even when two are in the same root row.
    let fixtures: &[(&str, &[&str])] = &[
        (r"|\log_{|}^{|}\left(|\right)|", &["", "b", "2", "x", ""]),
        (r"|\log^{|}\left(|\right)|", &["", "2", "x", ""]),
        ("|a_{|}^{|}|", &["", "i", "2", ""]),
        (r"|\sqrt[|]{|}|", &["", "3", "x", ""]),
    ];
    let actions = [
        Action::NextField(false),
        Action::NextField(true),
        Action::Insert("q".into()),
        Action::Undo,
        Action::Redo,
    ];
    for &(pattern, slots) in fixtures {
        let initial = Expected::new(slots);
        let mut seed = Editor::new();
        seed.import_latex(&initial.formula(pattern, "")).unwrap();
        seed.apply(Action::Move(Direction::Left, false)).unwrap();
        // A restored draft has no setup edits in its undo history.
        let seed = Editor::from_draft(&seed.draft().unwrap()).unwrap();
        let mut traces = VecDeque::from([Vec::<Action>::new()]);
        // Breadth-first enumeration reports a shortest failing trace for this fixture.
        while let Some(trace) = traces.pop_front() {
            for continuous in [false, true] {
                let mut editor = seed.clone();
                editor.structured_typing = continuous;
                let mut expected = initial.clone();
                let mut undo = Vec::new();
                let mut redo = Vec::new();
                for action in &trace {
                    match action {
                        Action::NextField(backward) => expected.tab(*backward),
                        Action::Insert(text) => {
                            undo.push(expected.clone());
                            redo.clear();
                            expected.slots[expected.field].insert_str(expected.offset, text);
                            expected.offset += text.len();
                        }
                        Action::Undo => {
                            if let Some(previous) = undo.pop() {
                                redo.push(std::mem::replace(&mut expected, previous));
                            }
                        }
                        Action::Redo => {
                            if let Some(next) = redo.pop() {
                                undo.push(std::mem::replace(&mut expected, next));
                            }
                        }
                        _ => unreachable!(),
                    }
                    editor.apply(action.clone()).unwrap_or_else(|error| {
                        panic!("{pattern}, continuous={continuous}, {trace:?}: {error}")
                    });
                }
                let observed = editor
                    .apply(Action::Insert("z".into()))
                    .and_then(|()| editor.latex());
                assert_eq!(
                    observed,
                    Ok(expected.formula(pattern, "z")),
                    "{pattern}, continuous={continuous}, actions={trace:?}"
                );
            }
            if trace.len() < 4 {
                for action in &actions {
                    let mut longer = trace.clone();
                    longer.push(action.clone());
                    traces.push_back(longer);
                }
            }
        }
    }
}

#[cfg(feature = "slint-ui")]
#[test]
fn tab_and_keypad_next_follow_the_host_typing_mode() {
    use slint::platform::Key;
    use slint_math_input::gui::MathInputController;

    for (source, steps_back, default_result, continuous_result) in [
        (
            r"\log^{2}\left(x\right)",
            1,
            r"\log^{2}q\left(x\right)",
            r"\log^{2}\left(qx\right)",
        ),
        (
            r"\log_{b}^{2}\left(x\right)",
            2,
            r"\log_{b}^{2}\left(qx\right)",
            r"\log_{b}^{q2}\left(x\right)",
        ),
        ("a_{i}^{2}", 1, "a_{i}^{2}q", "a_{i}^{q2}"),
        (r"\sqrt[3]{x}", 1, r"\sqrt[3]{x}q", r"\sqrt[3]{qx}"),
    ] {
        for continuous in [false, true] {
            for keypad in [false, true] {
                let mut input = MathInputController::new();
                input.editor.structured_typing = continuous;
                input.editor.import_latex(source).unwrap();
                input.keypad("key:left", false);
                for _ in 0..steps_back {
                    input.editor.apply(Action::NextField(true)).unwrap();
                }
                if keypad {
                    input.keypad("key:next", false);
                } else {
                    assert!(input.key(&char::from(Key::Tab).to_string(), false, false, false));
                }
                input.insert("q");
                assert_eq!(
                    input.editor.latex().unwrap(),
                    if continuous {
                        continuous_result
                    } else {
                        default_result
                    },
                    "{source}, continuous={continuous}, keypad={keypad}"
                );
            }
        }
    }
}
