use crate::{
    Editor, Error, Layout,
    layout::{Draw, Rect, font, math_char},
};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Rendered mathematical content. Selection is included; the blinking caret is separate.
#[derive(Clone, Debug)]
pub struct Rendered {
    /// Shared logical geometry for this image.
    pub layout: Layout,
    /// Width of the pixel buffer.
    pub pixel_width: u32,
    /// Height of the pixel buffer.
    pub pixel_height: u32,
    /// Premultiplied RGBA8 pixels, in row-major order.
    pub pixels: Vec<u8>,
}
impl Rendered {
    /// Write a PNG for diagnostics or visual regression inspection.
    pub fn save_png(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let size = tiny_skia::IntSize::from_wh(self.pixel_width, self.pixel_height)
            .ok_or_else(|| Error::new("Invalid image dimensions"))?;
        Pixmap::from_vec(self.pixels.clone(), size)
            .ok_or_else(|| Error::new("Invalid pixel buffer"))?
            .save_png(path)
            .map_err(|e| Error::new(e.to_string()))
    }
}
struct Outline {
    builder: PathBuilder,
}
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.builder.close();
    }
}
fn paint(color: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(color);
    p.anti_alias = true;
    p
}
fn rect(pixmap: &mut Pixmap, r: Rect, color: Color, scale: f32) {
    if let Some(rect) = tiny_skia::Rect::from_xywh(r.x, r.y, r.width.max(0.1), r.height.max(0.1)) {
        pixmap.fill_rect(
            rect,
            &paint(color),
            Transform::from_scale(scale, scale),
            None,
        );
    }
}
struct GlyphBitmap {
    pixels: Pixmap,
    left: i32,
    top: i32,
}
fn stamp(destination: &mut Pixmap, bitmap: &GlyphBitmap, x: i32, y: i32) {
    let source_width = bitmap.pixels.width() as usize;
    let destination_width = destination.width() as usize;
    let left = x.max(0);
    let right = (x + bitmap.pixels.width() as i32).min(destination.width() as i32);
    let top = y.max(0);
    let bottom = (y + bitmap.pixels.height() as i32).min(destination.height() as i32);
    if left >= right || top >= bottom {
        return;
    }
    let source = bitmap.pixels.data();
    let target = destination.data_mut();
    let bytes = (right - left) as usize * 4;
    for row in top..bottom {
        let src = ((row - y) as usize * source_width + (left - x) as usize) * 4;
        let dst = (row as usize * destination_width + left as usize) * 4;
        for (input, output) in source[src..src + bytes]
            .as_chunks::<4>()
            .0
            .iter()
            .zip(target[dst..dst + bytes].as_chunks_mut::<4>().0.iter_mut())
        {
            match input[3] {
                0 => {}
                255 => output.copy_from_slice(input),
                alpha => {
                    let inverse = 255 - u16::from(alpha);
                    for (s, d) in input.iter().zip(output) {
                        *d = (u16::from(*s) + (u16::from(*d) * inverse + 127) / 255) as u8;
                    }
                }
            }
        }
    }
}
fn glyph_bitmap(
    glyph: ttf_parser::GlyphId,
    factor: f32,
    phase_x: u8,
    phase_y: u8,
    ink: Color,
) -> Option<GlyphBitmap> {
    let mut outline = Outline {
        builder: PathBuilder::new(),
    };
    font().outline_glyph(glyph, &mut outline)?;
    let path = outline.builder.finish()?;
    let bounds = path.bounds();
    let dx = f32::from(phase_x) / 8.0;
    let dy = f32::from(phase_y) / 8.0;
    let left = (bounds.left() * factor + dx).floor() as i32 - 1;
    let top = (-bounds.bottom() * factor + dy).floor() as i32 - 1;
    let right = (bounds.right() * factor + dx).ceil() as i32 + 1;
    let bottom = (-bounds.top() * factor + dy).ceil() as i32 + 1;
    let mut pixels = Pixmap::new((right - left) as u32, (bottom - top) as u32)?;
    pixels.fill_path(
        &path,
        &paint(ink),
        FillRule::Winding,
        Transform::from_row(factor, 0.0, 0.0, -factor, dx - left as f32, dy - top as f32),
        None,
    );
    Some(GlyphBitmap { pixels, left, top })
}
pub(crate) fn render(editor: &Editor, scale: f32) -> Result<Rendered, Error> {
    if !scale.is_finite() || !(0.5..=4.0).contains(&scale) {
        return Err(Error::new("Pixel density must be between 0.5 and 4"));
    }
    let layout = editor.layout();
    let width = (layout.width * scale).ceil() as u32;
    let height = (layout.height * scale).ceil() as u32;
    if width > 32768 || height > 32768 || u64::from(width) * u64::from(height) > 32 * 1024 * 1024 {
        return Err(Error::new(
            "Formula image exceeds the rendering budget; the draft can still be saved",
        ));
    }
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| Error::new("Could not allocate the formula image"))?;
    let ink = Color::from_rgba8(35, 39, 43, 255);
    if let Some((row, start, end)) = editor.selection_range() {
        for selection in layout.range_rects(row, start, end) {
            rect(
                &mut pixmap,
                selection,
                Color::from_rgba8(177, 213, 246, 200),
                scale,
            );
        }
    }
    let face = font();
    // Bound each glyph's raster work to its own small bitmap. Eighth-pixel
    // phases retain subpixel placement while letting repeated symbols reuse it.
    let mut glyphs = std::collections::HashMap::new();
    let mut cache_bytes = 0;
    for draw in &layout.draws {
        match draw {
            Draw::Text {
                text,
                x,
                baseline,
                size,
                italic,
            } => {
                let mut x = *x;
                for ch in text.chars() {
                    let glyph = face
                        .glyph_index(math_char(ch, *italic))
                        .or_else(|| face.glyph_index(ch))
                        .unwrap_or(ttf_parser::GlyphId(0));
                    let factor = size / face.units_per_em() as f32;
                    let px = (x * scale * 8.0).round() as i32;
                    let py = (baseline * scale * 8.0).round() as i32;
                    let phase_x = px.rem_euclid(8) as u8;
                    let phase_y = py.rem_euclid(8) as u8;
                    let bitmap = glyphs
                        .entry((glyph.0, (factor * scale).to_bits(), phase_x, phase_y))
                        .or_insert_with(|| {
                            let bitmap = glyph_bitmap(glyph, factor * scale, phase_x, phase_y, ink);
                            cache_bytes += bitmap.as_ref().map_or(0, |b| b.pixels.data().len());
                            bitmap
                        });
                    if let Some(bitmap) = bitmap {
                        stamp(
                            &mut pixmap,
                            bitmap,
                            px.div_euclid(8) + bitmap.left,
                            py.div_euclid(8) + bitmap.top,
                        );
                    }
                    if cache_bytes > 8 * 1024 * 1024 {
                        glyphs.clear();
                        cache_bytes = 0;
                    }
                    x += face
                        .glyph_hor_advance(glyph)
                        .unwrap_or(face.units_per_em() / 2) as f32
                        * factor;
                }
            }
            Draw::Line {
                x1,
                y1,
                x2,
                y2,
                width,
            } => {
                let mut path = PathBuilder::new();
                path.move_to(*x1, *y1);
                path.line_to(*x2, *y2);
                if let Some(path) = path.finish() {
                    pixmap.stroke_path(
                        &path,
                        &paint(ink),
                        &Stroke {
                            width: *width,
                            ..Stroke::default()
                        },
                        Transform::from_scale(scale, scale),
                        None,
                    );
                }
            }
            Draw::Placeholder(r) => {
                let mut path = PathBuilder::new();
                if let Some(bounds) = tiny_skia::Rect::from_xywh(r.x, r.y, r.width, r.height) {
                    path.push_rect(bounds);
                }
                if let Some(path) = path.finish() {
                    pixmap.stroke_path(
                        &path,
                        &paint(Color::from_rgba8(159, 171, 183, 140)),
                        &Stroke {
                            width: 1.0,
                            ..Stroke::default()
                        },
                        Transform::from_scale(scale, scale),
                        None,
                    );
                }
            }
            Draw::Delimiter { ch, rect: r, ghost } => {
                let mut path = PathBuilder::new();
                let (l, t, w, h) = (r.x, r.y, r.width, r.height);
                match ch {
                    '(' => {
                        path.move_to(l + w, t);
                        path.cubic_to(
                            l - w * 0.2,
                            t + h * 0.2,
                            l - w * 0.2,
                            t + h * 0.8,
                            l + w,
                            t + h,
                        );
                    }
                    ')' => {
                        path.move_to(l, t);
                        path.cubic_to(l + w * 1.2, t + h * 0.2, l + w * 1.2, t + h * 0.8, l, t + h);
                    }
                    '[' | '⌊' | '⌈' => {
                        if *ch != '⌊' {
                            path.move_to(l + w, t);
                            path.line_to(l + w * 0.2, t);
                        } else {
                            path.move_to(l + w * 0.2, t);
                        }
                        path.line_to(l + w * 0.2, t + h);
                        if *ch != '⌈' {
                            path.line_to(l + w, t + h);
                        }
                    }
                    ']' | '⌋' | '⌉' => {
                        if *ch != '⌋' {
                            path.move_to(l, t);
                            path.line_to(l + w * 0.8, t);
                        } else {
                            path.move_to(l + w * 0.8, t);
                        }
                        path.line_to(l + w * 0.8, t + h);
                        if *ch != '⌉' {
                            path.line_to(l, t + h);
                        }
                    }
                    '⟨' | '⟩' => {
                        let (a, b) = if *ch == '⟨' { (l + w, l) } else { (l, l + w) };
                        path.move_to(a, t);
                        path.line_to(b, t + h / 2.0);
                        path.line_to(a, t + h);
                    }
                    '‖' => {
                        for x in [l + w * 0.3, l + w * 0.7] {
                            path.move_to(x, t);
                            path.line_to(x, t + h);
                        }
                    }
                    '⏞' | '⏟' => {
                        let (outer, inner) = if *ch == '⏞' { (t + h, t) } else { (t, t + h) };
                        let mid = (outer + inner) / 2.0;
                        path.move_to(l, outer);
                        path.cubic_to(l, mid, l + w * 0.1, mid, l + w * 0.22, mid);
                        path.line_to(l + w * 0.4, mid);
                        path.cubic_to(l + w * 0.49, mid, l + w * 0.49, inner, l + w * 0.5, inner);
                        path.cubic_to(l + w * 0.51, inner, l + w * 0.51, mid, l + w * 0.6, mid);
                        path.line_to(l + w * 0.78, mid);
                        path.cubic_to(l + w * 0.9, mid, l + w, mid, l + w, outer);
                    }
                    '|' => {
                        path.move_to(l + w * 0.5, t);
                        path.line_to(l + w * 0.5, t + h);
                    }
                    '{' | '}' => {
                        let (a, b) = if *ch == '{' { (l + w, l) } else { (l, l + w) };
                        path.move_to(a, t);
                        path.cubic_to(b, t, b, t + h * 0.4, b, t + h * 0.45);
                        path.line_to((a + b) / 2.0, t + h / 2.0);
                        path.line_to(b, t + h * 0.55);
                        path.cubic_to(b, t + h * 0.6, b, t + h, a, t + h);
                    }
                    _ => {}
                }
                if let Some(path) = path.finish() {
                    pixmap.stroke_path(
                        &path,
                        &paint(if *ghost {
                            Color::from_rgba8(166, 178, 189, 150)
                        } else {
                            ink
                        }),
                        &Stroke {
                            width: 1.5,
                            ..Stroke::default()
                        },
                        Transform::from_scale(scale, scale),
                        None,
                    );
                }
            }
        }
    }
    Ok(Rendered {
        layout,
        pixel_width: width,
        pixel_height: height,
        pixels: pixmap.take(),
    })
}
