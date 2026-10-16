//! Complete adapter for the standard SMI window template.
//! Hosts can reuse this wiring or embed MathEditor in their own window.
use slint::ComponentHandle;
use slint_math_input::{Action, Editor, Template, gui::MathInputController};
use std::{cell::RefCell, rc::Rc};

#[allow(missing_docs)]
mod generated {
    slint::slint! { export { MathInputWindow } from "../../templates/default-window.slint"; }
}
pub use generated::MathInputWindow;
use generated::MathView;

fn refresh(ui: &MathInputWindow, controller: &MathInputController) {
    match controller.view(ui.window().scale_factor()) {
        Ok(view) => {
            ui.set_view(MathView {
                image: view.image,
                content_width: view.width,
                content_height: view.height,
                caret_x: view.caret.x,
                caret_y: view.caret.y,
                caret_height: view.caret.height,
                font_size: view.font_size,
                accessible_text: view.spoken.into(),
            });
            ui.set_latex_source(view.latex.into());
            ui.set_status(
                controller
                    .error
                    .as_ref()
                    .map_or_else(
                        || {
                            if controller.editor.latex().is_ok() {
                                "Complete expression · LaTeX export available".into()
                            } else {
                                "Draft · empty slots and ghost brackets are preserved".into()
                            }
                        },
                        ToString::to_string,
                    )
                    .into(),
            );
        }
        Err(error) => ui.set_status(error.to_string().into()),
    }
}
/// Construct the standard window, with optional host-chosen draft storage.
pub fn create_window(
    draft_path: Option<std::path::PathBuf>,
) -> Result<MathInputWindow, Box<dyn std::error::Error>> {
    let ui = MathInputWindow::new()?;
    let controller = Rc::new(RefCell::new(MathInputController::new()));
    controller
        .borrow_mut()
        .editor
        .import_latex(r"f(x)=\frac{\sin\left(x\right)}{1+x^{2}}")?;
    if let Some(path) = draft_path.as_ref().filter(|path| path.exists()) {
        match Editor::load_draft_from(path) {
            Ok(editor) => {
                ui.set_draft_source(editor.draft()?.into());
                controller.borrow_mut().editor = editor;
            }
            Err(error) => controller.borrow_mut().error = Some(error),
        }
    }
    refresh(&ui, &controller.borrow());
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_keypad(move |token, extend| {
            let mut state = state.borrow_mut();
            state.keypad(&token, extend);
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_typed(move |text| {
            let mut state = state.borrow_mut();
            state.insert(&text);
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_command(move |text, control, shift, alt| {
            let mut state = state.borrow_mut();
            let handled = state.key(&text, control, shift, alt);
            if handled && let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
            handled
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_seek(move |x, y, extend| {
            let mut state = state.borrow_mut();
            state.pointer(x, y, extend);
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_insert_template(move |name| {
            let action = match name.as_str() {
                "a/b" => Action::Template(Template::Fraction),
                "√" => Action::Template(Template::Root),
                "ⁿ√" => Action::Template(Template::IndexedRoot),
                "xⁿ" => Action::Template(Template::Power),
                "xₙ" => Action::Template(Template::Subscript),
                "( )" => Action::Template(Template::Parentheses),
                "|x|" => Action::Template(Template::Absolute),
                "Σ" => Action::Template(Template::Sum),
                "Π" => Action::Template(Template::Product),
                "∫" => Action::Template(Template::Integral),
                "[2×2]" => Action::Template(Template::Matrix {
                    rows: 2,
                    columns: 2,
                }),
                "Undo" => Action::Undo,
                "Redo" => Action::Redo,
                "Clear" => Action::Clear,
                _ => Action::Insert(name.to_string()),
            };
            let mut state = state.borrow_mut();
            state.error = state.editor.apply(action).err();
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_import_latex(move |text| {
            let mut state = state.borrow_mut();
            state.error = state.editor.import_latex(&text).err();
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        let draft_path = draft_path.clone();
        ui.on_save_draft(move || {
            if let Some(ui) = weak.upgrade() {
                match state.borrow().editor.draft() {
                    Ok(draft) => {
                        ui.set_draft_source(draft.into());
                        let message = if let Some(path) = &draft_path {
                            match state.borrow().editor.save_draft_to(path) {
                                Ok(()) => format!("Draft saved: {}", path.display()),
                                Err(error) => error.to_string(),
                            }
                        } else {
                            "Draft saved with cursor and incomplete slots".into()
                        };
                        ui.set_status(message.into());
                    }
                    Err(error) => ui.set_status(error.to_string().into()),
                }
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_restore_draft(move |text| {
            let mut state = state.borrow_mut();
            state.error = state.editor.restore_draft(&text).err();
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_zoom(move |size| {
            let mut state = state.borrow_mut();
            state.editor.font_size = size as f32;
            if let Some(ui) = weak.upgrade() {
                refresh(&ui, &state);
            }
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_submitted(move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_status(
                    match state.borrow().editor.latex() {
                        Ok(latex) => format!("Submit intent: {latex}"),
                        Err(error) => format!("Draft retained: {error}"),
                    }
                    .into(),
                );
            }
        });
    }
    ui.invoke_focus_editor();
    Ok(ui)
}

/// Launch the native demonstration application.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("local_artifacts/demo-draft.json");
    create_window(Some(path))?.run()?;
    Ok(())
}
