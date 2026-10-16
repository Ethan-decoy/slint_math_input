//! Embed the standard input and keyboard in a host-owned window.
#![cfg(feature = "desktop")]
use slint::{
    ComponentHandle, SharedString,
    platform::{
        Key, Platform, PointerEventButton, WindowAdapter, WindowEvent,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};
use slint_math_input::{
    Error,
    gui::{ClipboardAccess, MathInputController, ViewData},
};
use std::{cell::RefCell, rc::Rc};

#[allow(missing_docs)]
mod host {
    slint::slint! {
        import {MathEditor,MathView} from "../ui/smi.slint";
        export component Host inherits Window {
            width:900px;height:640px;background:#edf3f7;
            in-out property<MathView> view;
            in-out property<bool> expanded <=> editor.keypad-expanded;
            in-out property<int> page <=> editor.keypad-page;
            in-out property<int> category <=> editor.keypad-category;
            out property<bool> input-focused:editor.has-focus;
            in-out property<int> submissions:0;
            callback typed(string);
            callback command(string,bool,bool,bool)->bool;
            callback keypad(string,bool);
            callback seek(float,float,bool);
            public function focus-input(){editor.focus();}
            Text{x:20px;y:20px;text:"Host-owned page · embedded SMI";color:#263a47;font-size:20px;}
            editor:=MathEditor{x:20px;y:80px;width:860px;height:480px;view:root.view;
                edited(t)=>{root.typed(t);}command(t,c,s,a)=>{return root.command(t,c,s,a);}
                key(t,e)=>{root.keypad(t,e);}seek(x,y,e)=>{root.seek(x,y,e);}
                submitted=>{root.submissions+=1;}}
            Text{x:20px;y:590px;text:"The host owns this window, its controls and submission policy.";color:#526572;}
        }
    }
}
struct TestPlatform(Rc<MinimalSoftwareWindow>);
impl Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
#[derive(Default)]
struct MemoryClipboard(String);
impl ClipboardAccess for MemoryClipboard {
    fn read_text(&mut self) -> Result<String, Error> {
        Ok(self.0.clone())
    }
    fn write_text(&mut self, text: String) -> Result<(), Error> {
        self.0 = text;
        Ok(())
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
fn paint(adapter: &MinimalSoftwareWindow) -> Vec<slint::Rgb8Pixel> {
    slint::platform::update_timers_and_animations();
    let mut pixels = vec![slint::Rgb8Pixel::default(); 900 * 640];
    adapter.request_redraw();
    adapter.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, 900);
    });
    pixels
}
fn native_key(ui: &host::Host, text: SharedString) {
    ui.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    ui.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}
#[test]
fn standard_editor_embeds_with_keypad_focus_and_host_owned_submission() {
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(adapter.clone()))).unwrap();
    let ui = host::Host::new().unwrap();
    let state = Rc::new(RefCell::new(MathInputController::with_clipboard(
        MemoryClipboard::default(),
    )));
    ui.set_view(view(state.borrow().view(1.0).unwrap()));
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_typed(move |text| {
            let mut state = state.borrow_mut();
            state.insert(&text);
            weak.unwrap().set_view(view(state.view(1.0).unwrap()));
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_command(move |text, control, shift, alt| {
            let mut state = state.borrow_mut();
            let handled = state.key(&text, control, shift, alt);
            weak.unwrap().set_view(view(state.view(1.0).unwrap()));
            handled
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_keypad(move |text, extend| {
            let mut state = state.borrow_mut();
            state.keypad(&text, extend);
            assert!(state.error.is_none(), "{:?}", state.error);
            weak.unwrap().set_view(view(state.view(1.0).unwrap()));
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_seek(move |x, y, extend| {
            let mut state = state.borrow_mut();
            state.pointer(x, y, extend);
            weak.unwrap().set_view(view(state.view(1.0).unwrap()));
        });
    }
    ui.window().set_size(slint::PhysicalSize::new(900, 640));
    ui.show().unwrap();
    ui.invoke_focus_input();
    paint(&adapter);
    native_key(&ui, "1/".into());
    paint(&adapter);
    let position = slint::LogicalPosition::new(492.0, 480.0);
    ui.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    ui.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    ui.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    paint(&adapter);
    assert_eq!(state.borrow().editor.latex().unwrap(), r"\frac{1}{2}");
    assert!(ui.get_input_focused());
    native_key(&ui, Key::Tab.into());
    native_key(&ui, "+x^2".into());
    assert_eq!(state.borrow().editor.latex().unwrap(), r"\frac{1}{2}+x^{2}");
    native_key(&ui, Key::Return.into());
    assert_eq!(ui.get_submissions(), 1);
    let draft = state.borrow().editor.draft().unwrap();
    ui.set_page(2);
    ui.set_category(8);
    paint(&adapter);
    ui.set_expanded(false);
    paint(&adapter);
    assert_eq!(state.borrow().editor.draft().unwrap(), draft);
    ui.set_expanded(true);
    let pixels = paint(&adapter);
    let mut image = tiny_skia::Pixmap::new(900, 640).unwrap();
    for (target, pixel) in image
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(pixels)
    {
        target.copy_from_slice(&[pixel.r, pixel.g, pixel.b, 255]);
    }
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("local_artifacts/rendered");
    std::fs::create_dir_all(&output).unwrap();
    image.save_png(output.join("embedded-editor.png")).unwrap();
    ui.hide().unwrap();
}
