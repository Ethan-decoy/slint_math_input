//! Exercise actual Slint input callbacks and capture the native component using software rendering.
#![cfg(feature = "desktop")]
use slint::{
    ComponentHandle, Rgb8Pixel, SharedPixelBuffer,
    platform::{
        Key, Platform, WindowAdapter, WindowEvent,
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
    },
};
use std::rc::Rc;

#[allow(dead_code)]
#[path = "../examples/support/mod.rs"]
mod template;

struct TestPlatform(Rc<MinimalSoftwareWindow>);
impl Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
fn key(window: &slint::Window, text: slint::SharedString) {
    window.dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window.dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}

#[test]
fn slint_native_text_keys_restore_and_software_layout_work_together() {
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(adapter.clone()))).unwrap();
    let ui = template::create_window(None).unwrap();
    ui.window().set_size(slint::PhysicalSize::new(1080, 760));
    ui.show().unwrap();
    ui.invoke_focus_editor();
    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(1080, 760);
    adapter.draw_if_needed(|renderer| {
        renderer.render(buffer.make_mut_slice(), 1080);
    });
    ui.invoke_insert_template("Clear".into());
    ui.invoke_focus_editor();
    for ch in ["1", "/", "2"] {
        key(ui.window(), ch.into());
    }
    assert_eq!(ui.get_latex_source().as_str(), r"\frac{1}{2}");
    key(ui.window(), Key::Tab.into());
    key(ui.window(), "+x^2".into());
    assert_eq!(ui.get_latex_source().as_str(), r"\frac{1}{2}+x^{2}");
    key(ui.window(), Key::Return.into());
    assert!(ui.get_status().starts_with("Submit intent:"));
    ui.invoke_save_draft();
    let draft = ui.get_draft_source();
    ui.invoke_insert_template("Clear".into());
    ui.invoke_restore_draft(draft);
    assert_eq!(ui.get_latex_source().as_str(), r"\frac{1}{2}+x^{2}");
    ui.invoke_import_latex(r"f(x)=\frac{\sin\left(x\right)}{1+x^{2}}".into());
    slint::platform::update_timers_and_animations();
    assert!(!ui.get_draft_source().is_empty());
    ui.window().request_redraw();
    assert!(adapter.draw_if_needed(|renderer| {
        renderer.render(buffer.make_mut_slice(), 1080);
    }));
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("local_artifacts/rendered");
    std::fs::create_dir_all(&output).unwrap();
    let mut image = tiny_skia::Pixmap::new(1080, 760).unwrap();
    for (pixel, source) in image
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(buffer.as_slice())
    {
        pixel.copy_from_slice(&[source.r, source.g, source.b, 255]);
    }
    image.save_png(output.join("demo-window.png")).unwrap();
    let stored_draft = ui.get_draft_source();
    ui.set_details_visible(true);
    slint::platform::update_timers_and_animations();
    assert_eq!(ui.get_draft_source(), stored_draft);
    ui.set_details_visible(false);
    ui.set_keypad_page(2);
    ui.set_keypad_category(8);
    ui.window().dispatch_event(WindowEvent::Resized {
        size: slint::LogicalSize::new(760.0, 600.0),
    });
    ui.window().set_size(slint::PhysicalSize::new(760, 600));
    ui.window().request_redraw();
    let mut small_buffer = vec![Rgb8Pixel::default(); 760 * 600];
    assert!(adapter.draw_if_needed(|renderer| {
        renderer.render(&mut small_buffer, 760);
    }));
    let mut small_image = tiny_skia::Pixmap::new(760, 600).unwrap();
    for (target, pixel) in small_image
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(small_buffer)
    {
        target.copy_from_slice(&[pixel.r, pixel.g, pixel.b, 255]);
    }
    small_image
        .save_png(output.join("demo-keypad-small.png"))
        .unwrap();
    assert_eq!(ui.get_draft_source(), stored_draft);
    assert!(ui.get_latex_source().contains(r"\frac"));
    ui.hide().unwrap();
}
