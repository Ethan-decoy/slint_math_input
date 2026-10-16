//! Draft persistence and transaction boundaries.
use slint_math_input::{Action, Editor};

#[test]
fn disk_round_trip_and_atomic_replacement_preserve_a_half_finished_expression() {
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("local_artifacts/test_runs");
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!("draft-{}.json", std::process::id()));
    let mut editor = Editor::new();
    editor.apply(Action::Insert("(1/".into())).unwrap();
    editor.save_draft_to(&path).unwrap();
    let restored = Editor::load_draft_from(&path).unwrap();
    assert_eq!(restored.draft().unwrap(), editor.draft().unwrap());
    editor.apply(Action::Insert("2".into())).unwrap();
    editor.save_draft_to(&path).unwrap();
    assert_eq!(
        Editor::load_draft_from(&path).unwrap().draft().unwrap(),
        editor.draft().unwrap()
    );
    std::fs::remove_file(&path).unwrap();
}
#[test]
fn draft_restore_is_undoable_and_invalid_restore_is_atomic() {
    let mut source = Editor::new();
    source.apply(Action::Insert("1/".into())).unwrap();
    let mut target = Editor::new();
    target.apply(Action::Insert("abc".into())).unwrap();
    let before = target.draft().unwrap();
    target.restore_draft(&source.draft().unwrap()).unwrap();
    assert_eq!(target.draft().unwrap(), source.draft().unwrap());
    target.apply(Action::Undo).unwrap();
    assert_eq!(target.draft().unwrap(), before);
    assert!(target.restore_draft("{bad json}").is_err());
    assert_eq!(target.draft().unwrap(), before);
}
#[test]
fn spoken_summary_distinguishes_nested_slots() {
    let mut editor = Editor::new();
    editor.import_latex(r"\frac{x^{2}}{}").unwrap();
    let spoken = editor.spoken_text();
    assert!(spoken.contains("superscript"));
    assert!(spoken.contains("denominator blank"));
}

#[test]
fn deep_valid_drafts_restore_and_excessive_json_is_rejected_before_recursion() {
    let mut editor = Editor::new();
    editor
        .import_latex(&format!("{}x{}", r"\sqrt{".repeat(64), "}".repeat(64)))
        .unwrap();
    let draft = editor.draft().unwrap();
    assert_eq!(Editor::from_draft(&draft).unwrap().draft().unwrap(), draft);
    let too_deep = format!("{}0{}", "[".repeat(513), "]".repeat(513));
    assert!(
        Editor::from_draft(&too_deep)
            .unwrap_err()
            .message
            .contains("nesting limit")
    );
    let before = editor.draft().unwrap();
    assert!(editor.restore_draft(&(draft + " trailing")).is_err());
    assert_eq!(editor.draft().unwrap(), before);
}
