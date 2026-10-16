//! Behavior and interchange coverage for the expanded notation catalog.
use slint_math_input::{Action, Direction, Editor, Position, Template, catalog};
use std::collections::HashSet;

fn insert(editor: &mut Editor, text: &str) {
    editor.apply(Action::Insert(text.into())).unwrap();
}
fn move_to(editor: &mut Editor, direction: Direction) {
    editor.apply(Action::Move(direction, false)).unwrap();
}

#[test]
fn every_catalog_insertion_is_valid_undoable_and_draft_restorable() {
    let mut ids = HashSet::new();
    for item in catalog::entries() {
        assert!(ids.insert(&item.id), "duplicate {}", item.id);
        let mut editor = Editor::new();
        if matches!(item.action, Action::ResizeMatrix { .. }) {
            editor
                .apply(Action::Template(Template::Matrix {
                    rows: 2,
                    columns: 2,
                }))
                .unwrap();
        }
        if matches!(
            item.action,
            Action::Template(Template::Power | Template::Subscript | Template::Square)
        ) {
            insert(&mut editor, "x");
        }
        let before = editor.draft().unwrap();
        editor
            .insert_catalog(&item.id)
            .unwrap_or_else(|error| panic!("{}: {error}", item.id));
        editor.validate().unwrap();
        let draft = editor.draft().unwrap();
        assert_eq!(
            Editor::from_draft(&draft).unwrap().draft().unwrap(),
            draft,
            "{}",
            item.id
        );
        let preview = editor.latex_preview();
        let mut imported = Editor::new();
        imported
            .import_latex(&preview)
            .unwrap_or_else(|error| panic!("{}: {preview}: {error}", item.id));
        assert_eq!(imported.latex_preview(), preview, "{}", item.id);
        let rendered = editor.render(1.0).unwrap();
        assert!(rendered.layout.width.is_finite() && rendered.layout.height.is_finite());
        editor.apply(Action::Undo).unwrap();
        assert_eq!(editor.draft().unwrap(), before, "undo {}", item.id);
        editor.apply(Action::Redo).unwrap();
        assert_eq!(editor.draft().unwrap(), draft, "redo {}", item.id);
    }
}

#[test]
fn limit_conditions_are_centered_editable_and_leave_for_the_function_body() {
    let mut editor = Editor::new();
    editor.insert_catalog("lim").unwrap();
    let condition = editor.cursor().row;
    assert!(editor.latex().is_err());
    insert(&mut editor, "x->0");
    assert_eq!(editor.latex().unwrap(), r"\lim_{x\to 0}");
    let draft: serde_json::Value = serde_json::from_str(&editor.draft().unwrap()).unwrap();
    let base = draft["state"]["root"]["nodes"][0]["base"]["id"]
        .as_u64()
        .unwrap();
    let layout = editor.layout();
    let base_start = layout
        .caret(Position {
            row: base,
            offset: 0,
        })
        .unwrap()
        .rect;
    let base_end = layout
        .caret(Position {
            row: base,
            offset: 1,
        })
        .unwrap()
        .rect;
    let lower_start = layout
        .caret(Position {
            row: condition,
            offset: 0,
        })
        .unwrap()
        .rect;
    let lower_end = layout
        .caret(Position {
            row: condition,
            offset: 3,
        })
        .unwrap()
        .rect;
    assert!(((base_start.x + base_end.x) - (lower_start.x + lower_end.x)).abs() < 2.0);
    assert!(lower_start.y > base_start.y + base_start.height * 0.6);
    editor.apply(Action::Escape(Direction::Right)).unwrap();
    insert(&mut editor, "f(x)");
    assert_eq!(editor.latex().unwrap(), r"\lim_{x\to 0}f\left(x\right)");
    move_to(&mut editor, Direction::Left);
    editor.validate().unwrap();
}

#[test]
fn named_functions_use_standard_commands_or_operatorname_and_extend_prefixes() {
    for (name, expected) in [
        ("limsup", r"\limsup"),
        ("liminf", r"\liminf"),
        ("arccsch", r"\operatorname{arccsch}"),
        ("nCr", r"\operatorname{nCr}"),
        ("mean", r"\operatorname{mean}"),
        ("floor", r"\operatorname{floor}"),
        ("mod", r"\operatorname{mod}"),
        ("normaldist", r"\operatorname{normaldist}"),
    ] {
        let mut editor = Editor::new();
        insert(&mut editor, name);
        assert_eq!(editor.latex().unwrap(), expected, "typed {name}");
        let mut restored = Editor::new();
        restored.import_latex(expected).unwrap();
        assert_eq!(restored.latex().unwrap(), expected);
    }
}

#[test]
fn binomial_navigation_selection_and_unwrap_preserve_arguments() {
    let mut editor = Editor::new();
    editor.insert_catalog("binomial").unwrap();
    insert(&mut editor, "n");
    move_to(&mut editor, Direction::Right);
    insert(&mut editor, "k");
    assert_eq!(editor.latex().unwrap(), r"\binom{n}{k}");
    editor.apply(Action::Home(false)).unwrap();
    editor.apply(Action::Backspace).unwrap();
    assert_eq!(editor.latex().unwrap(), "nk");
    editor.apply(Action::Undo).unwrap();
    assert_eq!(editor.latex().unwrap(), r"\binom{n}{k}");
    editor.apply(Action::SelectAll).unwrap();
    assert_eq!(editor.copy_latex(), r"\binom{n}{k}");
}

#[test]
fn common_latex_structures_and_symbols_canonicalize_without_losing_content() {
    let corpus = [
        r"\lim_{x\to0}\frac{\sin x}{x}",
        r"\limsup_{n\to\infty}a_n",
        r"\frac{\partial^2 f}{\partial x^2}",
        r"\binom{n}{k}",
        r"\left\lfloor\frac{x}{2}\right\rfloor",
        r"\lceil x\rceil",
        r"\left\lVert x\right\rVert",
        r"\langle u,v\rangle",
        r"\begin{pmatrix}a&b\\c&d\end{pmatrix}",
        r"\begin{vmatrix}a&b\\c&d\end{vmatrix}",
        r"\begin{Vmatrix}a&b\\c&d\end{Vmatrix}",
        r"\begin{Bmatrix}a&b\\c&d\end{Bmatrix}",
        r"\begin{cases}-x&x<0\\x&x\ge0\end{cases}",
        r"\begin{aligned}a&=b+c\\d&=e\end{aligned}",
        r"\vec{AB}+\widehat{x}+\widetilde{y}+\dot{x}+\ddot{y}",
        r"\overbrace{a+b}^{n}+\underbrace{x+y}_{m}",
        r"\overset{a}{\rightarrow}+\underset{x\to0}{f(x)}",
        r"\int f(x)dx+\oint_{C}f(z)dz+\iint_{D}f(x,y)",
        r"\sum_{\substack{i=1\\j=2}}^{n}a_{ij}",
        r"\mathbb{R}\supseteq\mathbb{Q},\mathcal{F}(\alpha,\varepsilon,\varphi)=\Phi+\Psi",
        r"\forall x\in A\subseteq B\Rightarrow x\notin\emptyset",
        r"f'(x)+f''(x)",
        r"\mathbf{AB12}+\mathbb{R^{n}}+\mathcal{FG}+\mathfrak{gh}+\mathsf{xyz}+\mathtt{a01}",
        r"\mathbf{A+\mathbb{B}}",
        r"\overset{\text{def}}{=}\quad\underset{n\to\infty}{\longrightarrow}",
        r"\left(x\rightarrow y\right)",
        r"\lim\nolimits_{x\to0}f(x)+\int\limits_{0}^{1}f(x)dx",
        r"\operatorname*{argmax}_{x}f(x)",
    ];
    for source in corpus {
        let mut editor = Editor::new();
        editor
            .import_latex(source)
            .unwrap_or_else(|e| panic!("{source}: {e}"));
        let canonical = editor.latex().unwrap_or_else(|e| panic!("{source}: {e}"));
        let mut other = Editor::new();
        other
            .import_latex(&canonical)
            .unwrap_or_else(|e| panic!("{canonical}: {e}"));
        assert_eq!(other.latex().unwrap(), canonical, "{source}");
        assert_eq!(other.spoken_text(), editor.spoken_text(), "{source}");
        let restored = Editor::from_draft(&editor.draft().unwrap()).unwrap();
        assert_eq!(restored.latex().unwrap(), canonical);
        for size in [18.0, 32.0, 60.0] {
            editor.font_size = size;
            let rendered = editor.render(1.0).unwrap();
            assert!(rendered.layout.width > 24.0);
        }
    }
}

#[test]
fn explicit_limit_placement_survives_exchange_and_changes_geometry() {
    let mut below = Editor::new();
    below.import_latex(r"\lim\limits_{x\to0}f(x)").unwrap();
    let mut side = Editor::new();
    side.import_latex(r"\lim\nolimits_{x\to0}f(x)").unwrap();
    assert!(below.layout().height > side.layout().height);
    assert!(below.latex().unwrap().contains("\\limits"));
    assert!(side.latex().unwrap().contains("\\nolimits"));
    assert_eq!(
        Editor::from_draft(&side.draft().unwrap())
            .unwrap()
            .latex()
            .unwrap(),
        side.latex().unwrap()
    );
}

#[test]
fn greek_variants_keep_their_standard_distinct_symbols() {
    let mut editor = Editor::new();
    editor
        .import_latex(r"\phi+\varphi+\epsilon+\varepsilon")
        .unwrap();
    assert_eq!(editor.spoken_text(), "ϕ plus φ plus ϵ plus ε");
    assert_eq!(
        editor.latex().unwrap(),
        r"\phi +\varphi +\epsilon +\varepsilon"
    );
}

#[test]
fn combined_scripts_and_explicit_alphabets_keep_their_intended_scope() {
    for text in ["x_1^2", "x^2_1"] {
        let mut editor = Editor::new();
        insert(&mut editor, text);
        assert_eq!(editor.latex().unwrap(), "x_{1}^{2}");
    }
    let mut editor = Editor::new();
    editor.insert_catalog("bold").unwrap();
    insert(&mut editor, "sinABC12");
    assert_eq!(editor.latex().unwrap(), r"\mathbf{sinABC12}");
}

#[test]
fn matrix_resize_preserves_content_and_rejects_loss_or_invalid_sizes() {
    let mut editor = Editor::new();
    editor
        .import_latex(r"\begin{pmatrix}1&2\\3&4\end{pmatrix}")
        .unwrap();
    move_to(&mut editor, Direction::Left);
    let original = editor.draft().unwrap();
    for (row_delta, column_delta) in [(-1, 0), (0, -1), (i32::MAX, 0), (0, i32::MIN)] {
        assert!(
            editor
                .apply(Action::ResizeMatrix {
                    row_delta,
                    column_delta
                })
                .is_err()
        );
        assert_eq!(editor.draft().unwrap(), original);
    }
    editor.insert_catalog("matrix-row-add").unwrap();
    assert!(editor.latex().is_err());
    editor.insert_catalog("matrix-row-remove").unwrap();
    assert_eq!(
        editor.latex().unwrap(),
        r"\begin{pmatrix}1&2\\3&4\end{pmatrix}"
    );
    editor.apply(Action::Undo).unwrap();
    editor.apply(Action::Undo).unwrap();
    assert_eq!(editor.draft().unwrap(), original);
}

#[test]
fn new_templates_capture_the_selection_and_fail_atomically() {
    let mut editor = Editor::new();
    insert(&mut editor, "xy");
    editor.apply(Action::SelectAll).unwrap();
    editor.insert_catalog("vector").unwrap();
    assert_eq!(editor.latex().unwrap(), r"\vec{xy}");
    editor.apply(Action::Home(false)).unwrap();
    editor.apply(Action::Backspace).unwrap();
    assert_eq!(editor.latex().unwrap(), "xy");
    editor.apply(Action::SelectAll).unwrap();
    editor.insert_catalog("derivative").unwrap();
    assert_eq!(editor.latex().unwrap(), r"\frac{d}{dx}xy");
    let before = editor.draft().unwrap();
    for action in [
        Action::Template(Template::Derivative {
            partial: false,
            order: 0,
        }),
        Action::Function("bad name".into()),
        Action::Template(Template::Limit("invalid".into())),
    ] {
        assert!(editor.apply(action).is_err());
        assert_eq!(editor.draft().unwrap(), before);
    }
    for source in [
        r"\int_a_b",
        r"\begin{cases}a&b&c\end{cases}",
        r"\begin{pmatrix}1&2\\3\end{pmatrix}",
    ] {
        assert!(editor.import_latex(source).is_err());
        assert_eq!(editor.draft().unwrap(), before);
    }
}
