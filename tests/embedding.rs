//! Independent host with two instances of the reusable Slint component.
#![cfg(feature = "desktop")]
use slint::{
    ComponentHandle, SharedString,
    platform::{
        Platform, PointerEventButton, WindowAdapter, WindowEvent,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};
use slint_math_input::{
    Action, Editor,
    gui::{MathInputController, ViewData},
};
use std::{cell::RefCell, rc::Rc};
#[allow(missing_docs)]
mod host {
    slint::slint! {
        import {MathInput,MathView} from "../ui/math-input.slint";
        export component Host inherits Window{
            width:800px;height:200px;
            in-out property<MathView> first-view;
            in-out property<MathView> second-view;
            callback first-text(string);callback second-text(string);
            callback first-key(string,bool,bool,bool)->bool;callback second-key(string,bool,bool,bool)->bool;
            callback second-seek(float,float,bool);
            public function focus-first(){first.focus();}public function focus-second(){second.focus();}
            first:=MathInput{x:0px;y:0px;width:400px;height:200px;view:root.first-view;edited(t)=>{root.first-text(t);}command(t,c,s,a)=>{return root.first-key(t,c,s,a);}}
            second:=MathInput{x:400px;y:0px;width:400px;height:200px;view:root.second-view;edited(t)=>{root.second-text(t);}command(t,c,s,a)=>{return root.second-key(t,c,s,a);}seek(x,y,e)=>{root.second-seek(x,y,e);}}
        }
    }
}
fn view(data: ViewData) -> host::MathView {
    host::MathView {
        image: data.image,
        content_width: data.width,
        content_height: data.height,
        caret_x: data.caret.x,
        caret_y: data.caret.y,
        caret_height: data.caret.height,
        font_size: data.font_size,
        accessible_text: data.spoken.into(),
    }
}
struct TestPlatform(Rc<MinimalSoftwareWindow>);
impl Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
fn key(ui: &host::Host, text: impl Into<SharedString>) {
    let text = text.into();
    ui.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    ui.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

#[test]
fn separate_host_instances_keep_focus_history_and_pointer_coordinates_independent() {
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(adapter.clone()))).unwrap();
    let ui = host::Host::new().unwrap();
    let first = Rc::new(RefCell::new(MathInputController::new()));
    let second = Rc::new(RefCell::new(MathInputController::new()));
    ui.set_first_view(view(first.borrow().view(1.0).unwrap()));
    ui.set_second_view(view(second.borrow().view(1.0).unwrap()));
    {
        let state = first.clone();
        let weak = ui.as_weak();
        ui.on_first_text(move |text| {
            let mut state = state.borrow_mut();
            state.insert(&text);
            weak.unwrap().set_first_view(view(state.view(1.0).unwrap()));
        });
    }
    {
        let state = second.clone();
        let weak = ui.as_weak();
        ui.on_second_text(move |text| {
            let mut state = state.borrow_mut();
            state.insert(&text);
            weak.unwrap()
                .set_second_view(view(state.view(1.0).unwrap()));
        });
    }
    {
        let state = first.clone();
        let weak = ui.as_weak();
        ui.on_first_key(move |t, c, s, a| {
            let mut state = state.borrow_mut();
            let result = state.key(&t, c, s, a);
            weak.unwrap().set_first_view(view(state.view(1.0).unwrap()));
            result
        });
    }
    {
        let state = second.clone();
        let weak = ui.as_weak();
        ui.on_second_key(move |t, c, s, a| {
            let mut state = state.borrow_mut();
            let result = state.key(&t, c, s, a);
            weak.unwrap()
                .set_second_view(view(state.view(1.0).unwrap()));
            result
        });
    }
    {
        let state = second.clone();
        let weak = ui.as_weak();
        ui.on_second_seek(move |x, y, e| {
            let mut state = state.borrow_mut();
            state.pointer(x, y, e);
            weak.unwrap()
                .set_second_view(view(state.view(1.0).unwrap()));
        });
    }
    ui.window().set_size(slint::PhysicalSize::new(800, 200));
    ui.show().unwrap();
    let mut pixels = vec![slint::Rgb8Pixel::default(); 800 * 200];
    adapter.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, 800);
    });
    ui.invoke_focus_first();
    for text in ["x", "^", "2"] {
        key(&ui, text);
    }
    assert_eq!(first.borrow().editor.latex().unwrap(), "x^{2}");
    ui.invoke_focus_second();
    for text in ["1", "/", "2"] {
        key(&ui, text);
    }
    assert_eq!(second.borrow().editor.latex().unwrap(), r"\frac{1}{2}");
    assert_eq!(first.borrow().editor.latex().unwrap(), "x^{2}");
    adapter.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, 800);
    });
    let reference = second.borrow().editor.layout();
    let target = reference
        .caret(second.borrow().editor.cursor())
        .unwrap()
        .rect;
    let point = slint::LogicalPosition::new(
        400.0 + 12.0 + target.x,
        (200.0 - reference.height) / 2.0 + target.y + target.height / 2.0,
    );
    ui.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    ui.window().dispatch_event(WindowEvent::PointerReleased {
        position: point,
        button: PointerEventButton::Left,
    });
    key(&ui, "3");
    assert_eq!(second.borrow().editor.latex().unwrap(), r"\frac{1}{23}");
    assert_eq!(first.borrow().editor.latex().unwrap(), "x^{2}");
    // Root-level Tab is delegated to the host's native focus traversal.
    first
        .borrow_mut()
        .editor
        .apply(Action::RootEnd(false))
        .unwrap();
    ui.set_first_view(view(first.borrow().view(1.0).unwrap()));
    ui.invoke_focus_first();
    key(&ui, slint::platform::Key::Tab);
    key(&ui, "4");
    assert_eq!(second.borrow().editor.latex().unwrap(), r"\frac{1}{234}");
    assert_eq!(first.borrow().editor.latex().unwrap(), "x^{2}");
    for scale in [1.5, 2.0, 3.0] {
        ui.window().dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale,
        });
        ui.window().dispatch_event(WindowEvent::Resized {
            size: slint::LogicalSize::new(800.0, 200.0),
        });
        ui.set_first_view(view(first.borrow().view(scale).unwrap()));
        ui.set_second_view(view(second.borrow().view(scale).unwrap()));
        let width = (800.0 * scale) as usize;
        let mut scaled_pixels = vec![slint::Rgb8Pixel::default(); width * (200.0 * scale) as usize];
        ui.window().request_redraw();
        assert!(adapter.draw_if_needed(|renderer| {
            renderer.render(&mut scaled_pixels, width);
        }));
        let geometry = second.borrow().editor.layout();
        let target = geometry
            .caret(second.borrow().editor.cursor())
            .unwrap()
            .rect;
        let point = slint::LogicalPosition::new(
            412.0 + target.x,
            (200.0 - geometry.height) / 2.0 + target.y + target.height / 2.0,
        );
        ui.window().dispatch_event(WindowEvent::PointerPressed {
            position: point,
            button: PointerEventButton::Left,
        });
        ui.window().dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
        key(&ui, "5");
        assert!(second.borrow().editor.latex().unwrap().ends_with("5}"));
        assert_eq!(first.borrow().editor.latex().unwrap(), "x^{2}");
    }
    let mut check = Editor::new();
    check
        .import_latex(&second.borrow().editor.latex().unwrap())
        .unwrap();
    check.validate().unwrap();
    ui.hide().unwrap();
}
