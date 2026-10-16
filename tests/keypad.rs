//! Mouse-only formula entry through the reusable Slint keypad and an independent host.
#![cfg(feature = "desktop")]
use slint::{
    ComponentHandle, Model, SharedString,
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
        import {MathInput,MathView,MathKeypad} from "../ui/smi.slint";
        import {Palette} from "std-widgets.slint";
        import {MathCatalog,CatalogSection} from "../ui/math-catalog.slint";
        export component Host inherits Window {
            width:1000px;height:500px;background:#f3f5f7;
            in-out property<MathView> view;
            in-out property<bool> expanded <=> pad.expanded;
            in-out property<bool> selecting <=> pad.selecting;
            in-out property<int> page <=> pad.page;
            in-out property<int> category <=> pad.category;
            out property<[CatalogSection]> catalog-sections:MathCatalog.sections;
            in-out property<length> catalog-offset <=> pad.catalog-scroll-position;
            in-out property<int> submissions:0;
            in-out property<string> font-probe;
            out property<bool> input-focused:input.has-focus;
            out property<bool> dark-theme:Palette.color-scheme==ColorScheme.dark;
            callback typed(string);
            callback command(string,bool,bool,bool)->bool;
            callback keypad(string,bool);
            public function focus-input(){input.focus();}
            public function set-dark-theme(dark:bool){Palette.color-scheme=dark?ColorScheme.dark:ColorScheme.light;}
            input:=MathInput{x:0px;y:0px;width:1000px;height:180px;view:root.view;
                edited(t)=>{root.typed(t);}command(t,c,s,a)=>{return root.command(t,c,s,a);}}
            pad:=MathKeypad{x:0px;y:200px;width:1000px;height:288px;
                key(t,e)=>{root.keypad(t,e);input.focus();}
                submitted=>{root.submissions+=1;input.focus();}
                focus-requested=>{input.focus();}}
            // Probe individual characters as well as whole buttons: a visible
            // prefix must not conceal a missing superscript or accented letter.
            Text{x:900px;y:10px;width:80px;height:60px;text:root.font-probe;
                font-family:"STIX Two Math";font-size:14px;color:#263a47;
                horizontal-alignment:center;vertical-alignment:center;}
        }
    }
}
struct TestPlatform(Rc<MinimalSoftwareWindow>);
impl Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}
struct MemoryClipboard(Rc<RefCell<String>>);
impl ClipboardAccess for MemoryClipboard {
    fn read_text(&mut self) -> Result<String, Error> {
        Ok(self.0.borrow().clone())
    }
    fn write_text(&mut self, text: String) -> Result<(), Error> {
        *self.0.borrow_mut() = text;
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
    paint_scaled(adapter, 1.0)
}
fn paint_scaled(adapter: &MinimalSoftwareWindow, scale: f32) -> Vec<slint::Rgb8Pixel> {
    slint::platform::update_timers_and_animations();
    let width = (1000.0 * scale) as usize;
    let height = (500.0 * scale) as usize;
    let mut pixels = vec![slint::Rgb8Pixel::default(); width * height];
    adapter.request_redraw();
    adapter.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, width);
    });
    pixels
}
fn click(ui: &host::Host, adapter: &MinimalSoftwareWindow, x: f32, y: f32) {
    let position = slint::LogicalPosition::new(x, y);
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
    paint(adapter);
}
fn cell(ui: &host::Host, adapter: &MinimalSoftwareWindow, col: usize, row: usize) {
    click(
        ui,
        adapter,
        58.0 + 98.2 * col as f32,
        313.0 + 47.5 * row as f32,
    );
}
fn select_category(ui: &host::Host, adapter: &MinimalSoftwareWindow, index: usize) {
    let count = ui.get_catalog_sections().row_count();
    let width = (976.0 - (count - 1) as f32 * 4.0) / count as f32;
    click(
        ui,
        adapter,
        12.0 + width / 2.0 + index as f32 * (width + 4.0),
        308.0,
    );
    assert_eq!(ui.get_category(), index as i32);
    assert_eq!(ui.get_catalog_offset(), 0.0);
    assert!(ui.get_input_focused());
}

fn catalog_sections(
    ui: &host::Host,
) -> Vec<(
    String,
    Vec<&'static slint_math_input::catalog::CatalogEntry>,
)> {
    ui.get_catalog_sections()
        .iter()
        .map(|section| {
            let keys = section
                .keys
                .iter()
                .map(|key| {
                    let id = key.value.strip_prefix("catalog:").unwrap();
                    let entry = slint_math_input::catalog::entry(id).unwrap();
                    assert_eq!(key.label, entry.label.as_str());
                    entry
                })
                .collect();
            (section.title.to_string(), keys)
        })
        .collect()
}
fn native_key(ui: &host::Host, text: &str) {
    let text: SharedString = text.into();
    ui.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    ui.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}
fn screenshot(adapter: &MinimalSoftwareWindow, name: &str) {
    let pixels = paint(adapter);
    let mut image = tiny_skia::Pixmap::new(1000, 500).unwrap();
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
    image.save_png(output.join(name)).unwrap();
}

fn luminance(pixel: slint::Rgb8Pixel) -> f64 {
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(pixel.r) + 0.7152 * linear(pixel.g) + 0.0722 * linear(pixel.b)
}

// Measure actual rendered glyphs, excluding borders and surrounding controls.
fn label_contrast(pixels: &[slint::Rgb8Pixel], x: usize, y: usize, half_width: usize) -> f64 {
    label_contrast_scaled(pixels, x, y, half_width, 1.0)
}

fn label_contrast_scaled(
    pixels: &[slint::Rgb8Pixel],
    x: usize,
    y: usize,
    half_width: usize,
    scale: f32,
) -> f64 {
    let physical = |value: usize| (value as f32 * scale) as usize;
    let width = physical(1000);
    let background = luminance(pixels[physical(y - 12) * width + physical(x)]);
    (physical(y - 8)..physical(y + 8))
        .flat_map(|row| {
            (physical(x - half_width)..physical(x + half_width)).map(move |col| row * width + col)
        })
        .map(|index| {
            let foreground = luminance(pixels[index]);
            (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
        })
        .fold(1.0, f64::max)
}

fn assert_calculus_symbols(ui: &host::Host, adapter: &MinimalSoftwareWindow, scale: f32) {
    ui.set_page(2);
    ui.set_category(2);
    paint_scaled(adapter, scale);
    ui.set_catalog_offset(-88.0);
    let pixels = paint_scaled(adapter, scale);
    let invisible: Vec<_> = [
        ("∬", 582, 349),
        ("∭", 746, 349),
        ("⋃", 910, 349),
        ("⋂", 90, 393),
        ("∐", 254, 393),
        ("∇", 910, 393),
    ]
    .into_iter()
    .filter_map(|(label, x, y)| {
        let contrast = label_contrast_scaled(&pixels, x, y, 20, scale);
        (contrast < 4.5).then(|| format!("{label}: {contrast:.2}:1"))
    })
    .collect();
    assert!(
        invisible.is_empty(),
        "Invisible catalog labels at {scale}x: {invisible:?}"
    );
}

fn assert_entire_catalog_visible(ui: &host::Host, adapter: &MinimalSoftwareWindow) {
    let sections = catalog_sections(ui);
    assert_eq!(
        sections
            .iter()
            .map(|(title, _)| title.as_str())
            .collect::<Vec<_>>(),
        [
            "结构",
            "常用",
            "微积分",
            "三角",
            "统计概率",
            "矩阵",
            "希腊字母",
            "关系集合",
            "符号常数"
        ]
    );
    let hidden = [
        "bold",
        "blackboard",
        "calligraphic",
        "fraktur",
        "italic",
        "sans-serif",
        "monospace",
        "underline",
        "fn-sign",
        "fn-factorial",
        "fn-asinh",
        "fn-acosh",
        "fn-atanh",
        "fn-asech",
        "fn-acsch",
        "fn-acoth",
    ];
    let entries: Vec<_> = sections.iter().flat_map(|(_, keys)| keys.iter()).collect();
    assert_eq!(
        entries.len(),
        slint_math_input::catalog::entries().len() - hidden.len()
    );
    assert!(
        entries
            .iter()
            .all(|entry| !hidden.contains(&entry.id.as_str()))
    );
    ui.set_page(2);
    let mut checked = 0;
    for (index, (_, section)) in sections.iter().enumerate() {
        select_category(ui, adapter, index);
        let height = section.len().div_ceil(6) * 44 - 6;
        for (row, keys) in section.chunks(6).enumerate() {
            let row_start = row * 44;
            let offset = row_start.min(height.saturating_sub(102));
            ui.set_catalog_offset(-(offset as f32));
            let pixels = paint(adapter);
            let y = 349 + row_start - offset;
            for (column, entry) in keys.iter().enumerate() {
                let x = (90.0 + 163.67 * column as f32) as usize;
                let contrast = label_contrast(&pixels, x, y, 60);
                if contrast < 2.0 {
                    screenshot(adapter, "catalog-missing-label.png");
                }
                assert!(
                    contrast >= 2.0,
                    "{} ({}) has no visible catalog label at ({x}, {y}), offset {} instead of -{offset}: {contrast:.2}:1",
                    entry.id,
                    entry.label,
                    ui.get_catalog_offset()
                );
                checked += 1;
            }
        }
        // Reclicking the active category jumps back to its first row.
        select_category(ui, adapter, index);
    }
    assert_eq!(checked, entries.len());
}

fn assert_catalog_characters_visible(ui: &host::Host, adapter: &MinimalSoftwareWindow) {
    // Exercise real Slint font resolution, including fallback for Chinese,
    // modifier letters and accented Latin characters absent from the math font.
    let sections = catalog_sections(ui);
    let characters: std::collections::BTreeSet<_> = sections
        .iter()
        .flat_map(|(_, keys)| keys.iter())
        .flat_map(|entry| entry.label.chars())
        .filter(|ch| !ch.is_whitespace())
        .collect();
    for ch in characters {
        ui.set_font_probe(ch.to_string().into());
        let pixels = paint(adapter);
        let contrast = label_contrast(&pixels, 940, 40, 25);
        // Thin antialiased strokes need not contain a fully covered pixel at
        // 14px. This checks glyph presence; button color contrast is separate.
        assert!(
            contrast >= 2.0,
            "Invisible catalog character {ch:?}: {contrast:.2}:1"
        );
    }
    ui.set_font_probe("".into());
}

fn assert_keypad_contrast(adapter: &MinimalSoftwareWindow, context: &str) {
    let pixels = paint(adapter);
    for (label, x, y, half_width) in [
        ("123", 44, 228, 16),
        ("ABC", 114, 228, 18),
        ("functions", 184, 228, 16),
        ("collapse", 952, 228, 28),
        ("select", 48, 268, 22),
        ("all", 118, 268, 16),
        ("copy", 180, 268, 16),
        ("cut", 242, 268, 16),
        ("paste", 304, 268, 16),
        ("undo", 366, 268, 16),
        ("redo", 428, 268, 16),
        ("clear", 490, 268, 16),
        ("variable", 58, 313, 10),
        ("submit", 942, 455, 16),
    ] {
        let contrast = label_contrast(&pixels, x, y, half_width);
        assert!(
            contrast >= 4.5,
            "{context}: {label} label contrast is only {contrast:.2}:1"
        );
    }
}

#[test]
fn mouse_keypad_enters_edits_and_exchanges_math_without_losing_native_focus() {
    let adapter = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(adapter.clone()))).unwrap();
    let ui = host::Host::new().unwrap();
    let clipboard = Rc::new(RefCell::new(String::new()));
    let controller = Rc::new(RefCell::new(MathInputController::with_clipboard(
        MemoryClipboard(clipboard.clone()),
    )));
    ui.set_view(view(controller.borrow().view(2.0).unwrap()));
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_keypad(move |token, extend| {
            let mut state = state.borrow_mut();
            state.keypad(&token, extend);
            assert!(state.error.is_none(), "{:?}", state.error);
            weak.unwrap().set_view(view(state.view(2.0).unwrap()));
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_typed(move |text| {
            let mut state = state.borrow_mut();
            state.insert(&text);
            weak.unwrap().set_view(view(state.view(2.0).unwrap()));
        });
    }
    {
        let state = controller.clone();
        let weak = ui.as_weak();
        ui.on_command(move |t, c, s, a| {
            let mut state = state.borrow_mut();
            let used = state.key(&t, c, s, a);
            weak.unwrap().set_view(view(state.view(2.0).unwrap()));
            used
        });
    }
    ui.window().set_size(slint::PhysicalSize::new(1000, 500));
    ui.show().unwrap();
    ui.invoke_focus_input();
    paint(&adapter);
    // Reproduce the reported row, then cover every label and host palette.
    for dark in [false, true] {
        ui.invoke_set_dark_theme(dark);
        assert_calculus_symbols(&ui, &adapter, 1.0);
        screenshot(&adapter, "keypad-calculus-symbols.png");
        assert_entire_catalog_visible(&ui, &adapter);
        assert_catalog_characters_visible(&ui, &adapter);
    }
    for scale in [1.5, 2.0] {
        ui.window().dispatch_event(WindowEvent::ScaleFactorChanged {
            scale_factor: scale,
        });
        ui.window().set_size(slint::PhysicalSize::new(
            (1000.0 * scale) as u32,
            (500.0 * scale) as u32,
        ));
        assert_calculus_symbols(&ui, &adapter, scale);
    }
    ui.window()
        .dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: 1.0 });
    ui.window().set_size(slint::PhysicalSize::new(1000, 500));
    ui.set_page(0);
    ui.set_catalog_offset(0.0);
    // The keypad must keep its light surface legible without changing the host palette.
    for dark in [false, true] {
        ui.invoke_set_dark_theme(dark);
        assert_eq!(ui.get_dark_theme(), dark);
        assert_keypad_contrast(&adapter, if dark { "dark host" } else { "light host" });
        let position = slint::LogicalPosition::new(366.0, 268.0);
        ui.window()
            .dispatch_event(WindowEvent::PointerMoved { position });
        assert_keypad_contrast(&adapter, "hovered undo");
        ui.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        assert_keypad_contrast(&adapter, "pressed undo");
        ui.window().dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        click(&ui, &adapter, 48.0, 268.0);
        assert!(ui.get_selecting());
        assert_keypad_contrast(&adapter, "checked selection");
        click(&ui, &adapter, 48.0, 268.0);
        for page in [1, 2, 0] {
            click(&ui, &adapter, 44.0 + 70.0 * page as f32, 228.0);
            assert_eq!(ui.get_page(), page);
            // Function and alphabet pages have different bottom-right keys.
            let pixels = paint(&adapter);
            for x in [44, 114, 184] {
                assert!(label_contrast(&pixels, x, 228, 16) >= 4.5);
            }
        }
        screenshot(
            &adapter,
            if dark {
                "keypad-dark-host.png"
            } else {
                "keypad-light-host.png"
            },
        );
    }
    // The local button styling must retain keyboard activation and focus return.
    for activate in [Key::Return, Key::Space] {
        ui.invoke_focus_input();
        native_key(&ui, &SharedString::from(Key::Tab));
        paint(&adapter);
        assert!(!ui.get_input_focused());
        native_key(&ui, &SharedString::from(activate));
        paint(&adapter);
        assert!(ui.get_input_focused());
        assert_eq!(ui.get_page(), 0);
        assert_eq!(controller.borrow().editor.latex_preview(), "");
    }
    // Click 1, fraction, 2, exit, +, x, square. No text callback is invoked directly.
    for (col, row) in [(4, 2), (7, 0), (5, 2), (9, 0), (7, 3), (0, 0), (2, 0)] {
        cell(&ui, &adapter, col, row);
    }
    assert_eq!(
        controller.borrow().editor.latex().unwrap(),
        r"\frac{1}{2}+x^{2}"
    );
    assert!(ui.get_input_focused());
    native_key(&ui, "+3");
    paint(&adapter);
    assert_eq!(
        controller.borrow().editor.latex().unwrap(),
        r"\frac{1}{2}+x^{2}+3"
    );
    // Select the final digit using the mouse selection latch and arrow, then replace it.
    click(&ui, &adapter, 48.0, 268.0);
    assert!(ui.get_selecting());
    cell(&ui, &adapter, 8, 0);
    cell(&ui, &adapter, 6, 0);
    assert_eq!(
        controller.borrow().editor.latex().unwrap(),
        r"\frac{1}{2}+x^{2}+9"
    );
    assert!(!ui.get_selecting());
    click(&ui, &adapter, 366.0, 268.0);
    assert!(controller.borrow().editor.latex().unwrap().ends_with("+3"));
    click(&ui, &adapter, 428.0, 268.0);
    assert!(controller.borrow().editor.latex().unwrap().ends_with("+9"));
    // All/copy/cut/paste use an isolated clipboard, preserving the real OS clipboard.
    click(&ui, &adapter, 118.0, 268.0);
    click(&ui, &adapter, 180.0, 268.0);
    assert_eq!(&*clipboard.borrow(), r"\frac{1}{2}+x^{2}+9");
    click(&ui, &adapter, 242.0, 268.0);
    assert_eq!(controller.borrow().editor.latex_preview(), "");
    click(&ui, &adapter, 304.0, 268.0);
    assert_eq!(
        controller.borrow().editor.latex().unwrap(),
        r"\frac{1}{2}+x^{2}+9"
    );
    cell(&ui, &adapter, 9, 3);
    assert_eq!(ui.get_submissions(), 1);
    assert!(ui.get_input_focused());
    screenshot(&adapter, "keypad-numeric.png");
    click(&ui, &adapter, 952.0, 228.0);
    assert!(!ui.get_expanded());
    let before = controller.borrow().editor.draft().unwrap();
    cell(&ui, &adapter, 4, 2);
    assert_eq!(controller.borrow().editor.draft().unwrap(), before);
    click(&ui, &adapter, 952.0, 228.0);
    assert!(ui.get_expanded());
    // Alphabet page and case switching affect subsequent mouse keys only.
    click(&ui, &adapter, 490.0, 268.0);
    click(&ui, &adapter, 114.0, 228.0);
    assert_eq!(ui.get_page(), 1);
    cell(&ui, &adapter, 0, 0);
    cell(&ui, &adapter, 0, 3);
    cell(&ui, &adapter, 1, 0);
    assert_eq!(controller.borrow().editor.latex().unwrap(), "qW");
    assert!(ui.get_input_focused());
    screenshot(&adapter, "keypad-letters.png");
    // Jump directly to trigonometry, then enter sin(x).
    click(&ui, &adapter, 490.0, 268.0);
    click(&ui, &adapter, 184.0, 228.0);
    assert_eq!(ui.get_page(), 2);
    let before_category = controller.borrow().editor.draft().unwrap();
    select_category(&ui, &adapter, 3);
    assert_eq!(controller.borrow().editor.draft().unwrap(), before_category);
    screenshot(&adapter, "catalog-scroll-probe.png");
    click(&ui, &adapter, 90.0, 349.0);
    click(&ui, &adapter, 44.0, 228.0);
    for (col, row) in [(0, 1), (0, 0), (1, 1)] {
        cell(&ui, &adapter, col, row);
    }
    assert_eq!(
        controller.borrow().editor.latex().unwrap(),
        r"\sin \left(x\right)"
    );
    click(&ui, &adapter, 184.0, 228.0);
    click(&ui, &adapter, 118.0, 268.0);
    let selected_draft = controller.borrow().editor.draft().unwrap();
    // Jump between distant categories with a real nonempty selection.
    for category in [5, 8, 2] {
        select_category(&ui, &adapter, category);
        assert_eq!(controller.borrow().editor.draft().unwrap(), selected_draft);
    }
    screenshot(&adapter, "keypad-functions.png");
    // The first catalog key inserts a real editable limit condition.
    click(&ui, &adapter, 490.0, 268.0);
    click(&ui, &adapter, 90.0, 349.0);
    assert!(controller.borrow().editor.latex().is_err());
    assert!(ui.get_input_focused());
    native_key(&ui, "x->0");
    paint(&adapter);
    // The fixed footer leaves the condition even while the catalog is scrolled.
    click(&ui, &adapter, 560.0, 457.0);
    native_key(&ui, "sin(x)/x");
    assert_eq!(
        controller.borrow().editor.latex().unwrap(),
        r"\lim_{x\to 0}\frac{\sin \left(x\right)}{x}"
    );
    assert!(ui.get_input_focused());
    screenshot(&adapter, "keypad-limit.png");
    let before_scroll = controller.borrow().editor.draft().unwrap();
    ui.window().dispatch_event(WindowEvent::PointerScrolled {
        position: slint::LogicalPosition::new(450.0, 370.0),
        delta_x: 0.0,
        delta_y: -120.0,
    });
    paint(&adapter);
    assert!(ui.get_catalog_offset() < 0.0);
    assert_eq!(controller.borrow().editor.draft().unwrap(), before_scroll);
    ui.hide().unwrap();
}
