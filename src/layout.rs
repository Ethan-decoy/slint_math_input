use crate::{
    Accent, MatrixStyle, Position,
    model::{LimitPlacement, Node, Row},
};

pub(crate) const FONT_DATA: &[u8] = include_bytes!("../assets/STIXTwoMath-Regular.ttf");
pub(crate) fn font() -> &'static ttf_parser::Face<'static> {
    static FONT: std::sync::OnceLock<ttf_parser::Face<'static>> = std::sync::OnceLock::new();
    FONT.get_or_init(|| ttf_parser::Face::parse(FONT_DATA, 0).expect("bundled STIX Two Math font"))
}
pub(crate) fn math_char(c: char, italic: bool) -> char {
    if italic && c.is_ascii_lowercase() {
        if c == 'h' {
            'ℎ'
        } else {
            char::from_u32(0x1d44e + c as u32 - 'a' as u32).unwrap_or(c)
        }
    } else if italic && c.is_ascii_uppercase() {
        char::from_u32(0x1d434 + c as u32 - 'A' as u32).unwrap_or(c)
    } else {
        c
    }
}
pub(crate) fn text_width(text: &str, size: f32, italic: bool) -> f32 {
    let face = font();
    text.chars()
        .map(|c| {
            face.glyph_index(math_char(c, italic))
                .or_else(|| face.glyph_index(c))
                .and_then(|id| face.glyph_hor_advance(id))
                .unwrap_or(face.units_per_em() / 2) as f32
                / face.units_per_em() as f32
                * size
        })
        .sum()
}

/// A logical-pixel rectangle shared by rendering, selection, and hit testing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}
impl Rect {
    fn shifted(self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            ..self
        }
    }
}
/// A selectable insertion position in a laid-out expression.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caret {
    /// Corresponding editing position.
    pub position: Position,
    /// Geometry of the insertion bar.
    pub rect: Rect,
    pub(crate) depth: usize,
}
#[derive(Clone, Debug)]
pub(crate) enum Draw {
    Text {
        text: String,
        x: f32,
        baseline: f32,
        size: f32,
        italic: bool,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
    },
    Delimiter {
        ch: char,
        rect: Rect,
        ghost: bool,
    },
    Placeholder(Rect),
}
impl Draw {
    fn shift(&mut self, x: f32, y: f32) {
        match self {
            Self::Text {
                x: px, baseline, ..
            } => {
                *px += x;
                *baseline += y;
            }
            Self::Line { x1, y1, x2, y2, .. } => {
                *x1 += x;
                *x2 += x;
                *y1 += y;
                *y2 += y;
            }
            Self::Delimiter { rect, .. } | Self::Placeholder(rect) => *rect = rect.shifted(x, y),
        }
    }
}
#[derive(Clone, Debug)]
struct Span {
    row: u64,
    index: usize,
    rect: Rect,
}
#[derive(Clone, Debug, Default)]
struct BoxLayout {
    width: f32,
    ascent: f32,
    descent: f32,
    draws: Vec<Draw>,
    carets: Vec<Caret>,
    spans: Vec<Span>,
}
impl BoxLayout {
    fn place(&mut self, mut child: Self, x: f32, y: f32) {
        for draw in &mut child.draws {
            draw.shift(x, y);
        }
        for caret in &mut child.carets {
            caret.rect = caret.rect.shifted(x, y);
        }
        for span in &mut child.spans {
            span.rect = span.rect.shifted(x, y);
        }
        self.draws.extend(child.draws);
        self.carets.extend(child.carets);
        self.spans.extend(child.spans);
    }
}
/// A complete formula's layout. Coordinates refer to the transparent image origin.
#[derive(Clone, Debug)]
pub struct Layout {
    /// Total logical width, including outer padding.
    pub width: f32,
    /// Total logical height, including outer padding.
    pub height: f32,
    /// All insertion boundaries, including empty structural slots.
    pub carets: Vec<Caret>,
    pub(crate) draws: Vec<Draw>,
    spans: Vec<Span>,
}
impl Layout {
    pub(crate) fn build(root: &Row, font_size: f32) -> Self {
        let size = if font_size.is_finite() {
            font_size.clamp(12.0, 144.0)
        } else {
            32.0
        };
        let mut outer = BoxLayout::default();
        let child = row_layout(root, size, 0);
        let width = child.width + 24.0;
        let height = (child.ascent + child.descent + 24.0).max(size + 28.0);
        let baseline = child.ascent + (height - child.ascent - child.descent) / 2.0;
        outer.place(child, 12.0, baseline);
        Self {
            width,
            height,
            carets: outer.carets,
            draws: outer.draws,
            spans: outer.spans,
        }
    }
    /// Find the insertion bar for a validated document position.
    pub fn caret(&self, position: Position) -> Option<&Caret> {
        self.carets.iter().find(|c| c.position == position)
    }
    /// Return the nearest insertion position, preferring nearby nested slots.
    pub fn hit_test(&self, x: f32, y: f32) -> Position {
        self.carets
            .iter()
            .min_by(|a, b| score(a, x, y).total_cmp(&score(b, x, y)))
            .expect("root has a caret")
            .position
    }
    /// Selection rectangles for a range within one row.
    pub fn range_rects(&self, row: u64, start: usize, end: usize) -> Vec<Rect> {
        let spans: Vec<_> = self
            .spans
            .iter()
            .filter(|s| s.row == row && s.index >= start && s.index < end)
            .collect();
        if spans.is_empty() {
            return vec![];
        }
        let left = spans.iter().map(|s| s.rect.x).fold(f32::INFINITY, f32::min);
        let top = spans.iter().map(|s| s.rect.y).fold(f32::INFINITY, f32::min);
        let right = spans
            .iter()
            .map(|s| s.rect.x + s.rect.width)
            .fold(f32::NEG_INFINITY, f32::max);
        let bottom = spans
            .iter()
            .map(|s| s.rect.y + s.rect.height)
            .fold(f32::NEG_INFINITY, f32::max);
        vec![Rect {
            x: left,
            y: top - 2.0,
            width: right - left,
            height: bottom - top + 4.0,
        }]
    }
    pub(crate) fn nearest_in_row(&self, row: u64, x: f32) -> Option<Position> {
        self.carets
            .iter()
            .filter(|c| c.position.row == row)
            .min_by(|a, b| (a.rect.x - x).abs().total_cmp(&(b.rect.x - x).abs()))
            .map(|c| c.position)
    }
    pub(crate) fn vertical(&self, position: Position, down: bool, x: f32) -> Option<Position> {
        let current = self.caret(position)?;
        let center = current.rect.y + current.rect.height / 2.0;
        self.carets
            .iter()
            .filter(|c| {
                c.position.row != position.row
                    && if down {
                        c.rect.y + c.rect.height / 2.0 > center + 3.0
                    } else {
                        c.rect.y + c.rect.height / 2.0 < center - 3.0
                    }
            })
            .min_by(|a, b| {
                let weight = |c: &Caret| {
                    (c.rect.x - x).abs() + (c.rect.y + c.rect.height / 2.0 - center).abs() * 1.8
                };
                weight(a).total_cmp(&weight(b))
            })
            .map(|c| c.position)
    }
}
fn score(c: &Caret, x: f32, y: f32) -> f32 {
    let center = c.rect.y + c.rect.height / 2.0;
    (c.rect.x - x).abs() + (center - y).abs() * 1.5 - c.depth as f32 * 0.03
}
fn row_layout(row: &Row, size: f32, depth: usize) -> BoxLayout {
    let mut result = BoxLayout {
        ascent: size * 0.78,
        descent: size * 0.22,
        ..BoxLayout::default()
    };
    if row.nodes.is_empty() {
        result.width = size * 0.58;
        result.draws.push(Draw::Placeholder(Rect {
            x: size * 0.08,
            y: -size * 0.65,
            width: size * 0.42,
            height: size * 0.7,
        }));
        result.carets.push(Caret {
            position: Position {
                row: row.id,
                offset: 0,
            },
            rect: Rect {
                x: size * 0.29,
                y: -size * 0.78,
                width: 1.5,
                height: size,
            },
            depth,
        });
        return result;
    }
    for (index, node) in row.nodes.iter().enumerate() {
        let pad = if node.is_boundary() {
            size * 0.13
        } else {
            size * 0.015
        };
        result.carets.push(Caret {
            position: Position {
                row: row.id,
                offset: index,
            },
            rect: Rect {
                x: result.width,
                y: -size * 0.78,
                width: 1.5,
                height: size,
            },
            depth,
        });
        let child = node_layout(node, size, depth + 1);
        result.ascent = result.ascent.max(child.ascent);
        result.descent = result.descent.max(child.descent);
        result.spans.push(Span {
            row: row.id,
            index,
            rect: Rect {
                x: result.width,
                y: -child.ascent,
                width: child.width + pad * 2.0,
                height: child.ascent + child.descent,
            },
        });
        let width = child.width;
        let x = result.width + pad;
        result.place(child, x, 0.0);
        result.width += width + pad * 2.0;
    }
    result.carets.push(Caret {
        position: Position {
            row: row.id,
            offset: row.nodes.len(),
        },
        rect: Rect {
            x: result.width,
            y: -size * 0.78,
            width: 1.5,
            height: size,
        },
        depth,
    });
    result
}
fn leaf(text: &str, size: f32, italic: bool) -> BoxLayout {
    BoxLayout {
        width: text_width(text, size, italic),
        ascent: size * 0.78,
        descent: size * 0.22,
        draws: vec![Draw::Text {
            text: text.into(),
            x: 0.0,
            baseline: 0.0,
            size,
            italic,
        }],
        ..BoxLayout::default()
    }
}
fn node_layout(node: &Node, size: f32, depth: usize) -> BoxLayout {
    let mut result = BoxLayout::default();
    let small = (size * 0.68).max(10.0);
    match node {
        Node::Symbol { text } => return leaf(text, size, true),
        Node::Function { name } => {
            let mut b = leaf(name, size, false);
            b.width += size * 0.08;
            return b;
        }
        Node::Text { text } => return leaf(text, size, false),
        Node::Styled { alphabet, body } => {
            let mut styled = body.clone();
            alphabet.apply(&mut styled);
            return row_layout(&styled, size, depth);
        }
        Node::Fraction {
            numerator,
            denominator,
        }
        | Node::Binomial {
            upper: numerator,
            lower: denominator,
        } => {
            let binomial = matches!(node, Node::Binomial { .. });
            let n = row_layout(numerator, (size * 0.88).max(11.0), depth);
            let d = row_layout(denominator, (size * 0.88).max(11.0), depth);
            let pad = size * 0.16;
            let gap = size * 0.15;
            let axis = -size * 0.22;
            result.width = n.width.max(d.width) + 2.0 * pad;
            let ny = axis - gap - n.descent;
            let dy = axis + gap + d.ascent;
            result.ascent = n.ascent - ny;
            result.descent = dy + d.descent;
            let nx = (result.width - n.width) / 2.0;
            let dx = (result.width - d.width) / 2.0;
            result.place(n, nx, ny);
            result.place(d, dx, dy);
            if !binomial {
                result.draws.push(Draw::Line {
                    x1: 0.0,
                    y1: axis,
                    x2: result.width,
                    y2: axis,
                    width: (size * 0.045).max(1.0),
                });
            } else {
                let bracket = size * 0.24;
                for draw in &mut result.draws {
                    draw.shift(bracket, 0.0);
                }
                for caret in &mut result.carets {
                    caret.rect.x += bracket;
                }
                for span in &mut result.spans {
                    span.rect.x += bracket;
                }
                result.width += bracket * 2.0;
                for (ch, x) in [('(', 0.0), (')', result.width - bracket)] {
                    result.draws.push(Draw::Delimiter {
                        ch,
                        rect: Rect {
                            x,
                            y: -result.ascent,
                            width: bracket,
                            height: result.ascent + result.descent,
                        },
                        ghost: false,
                    });
                }
            }
        }
        Node::Script {
            placement,
            base,
            sub,
            sup,
        } => {
            if *placement == LimitPlacement::AboveBelow
                || (*placement == LimitPlacement::Automatic && base.centers_limits())
            {
                return centered_stack(base, sub, sup, size, depth);
            }
            let base = row_layout(base, size, depth);
            let x = base.width + size * 0.03;
            result.width = base.width;
            result.ascent = base.ascent;
            result.descent = base.descent;
            result.place(base, 0.0, 0.0);
            if let Some(sup) = sup {
                let s = row_layout(sup, small, depth);
                let y = -size * 0.58 - s.descent;
                result.width = result.width.max(x + s.width);
                result.ascent = result.ascent.max(s.ascent - y);
                result.place(s, x, y);
            }
            if let Some(sub) = sub {
                let s = row_layout(sub, small, depth);
                let y = size * 0.22 + s.ascent * 0.55;
                result.width = result.width.max(x + s.width);
                result.descent = result.descent.max(y + s.descent);
                result.place(s, x, y);
            }
        }
        Node::Stacked { base, under, over } => {
            return centered_stack(base, under, over, size, depth);
        }
        Node::Accent { accent, body } => {
            let body = row_layout(body, size, depth);
            result.width = body.width.max(size * 0.35);
            result.ascent = body.ascent;
            result.descent = body.descent;
            let below = matches!(accent, Accent::Underline | Accent::Underbrace);
            let height = if matches!(accent, Accent::Bar | Accent::Underline) {
                size * 0.08
            } else {
                size * 0.24
            };
            let y = if below {
                body.descent + size * 0.12
            } else {
                -body.ascent - size * 0.12 - height
            };
            let w = result.width;
            if below {
                result.descent = y + height;
            } else {
                result.ascent = -y;
            }
            let line = |x1, y1, x2, y2| Draw::Line {
                x1,
                y1,
                x2,
                y2,
                width: (size * 0.04).max(1.0),
            };
            match accent {
                Accent::Bar | Accent::Underline => {
                    result
                        .draws
                        .push(line(0.0, y + height / 2.0, w, y + height / 2.0))
                }
                Accent::Hat => {
                    result.draws.push(line(0.0, y + height, w / 2.0, y));
                    result.draws.push(line(w / 2.0, y, w, y + height));
                }
                Accent::Vector => {
                    result
                        .draws
                        .push(line(0.0, y + height / 2.0, w, y + height / 2.0));
                    result
                        .draws
                        .push(line(w - size * 0.2, y, w, y + height / 2.0));
                    result
                        .draws
                        .push(line(w - size * 0.2, y + height, w, y + height / 2.0));
                }
                Accent::Dot | Accent::DoubleDot => {
                    let text = if *accent == Accent::Dot { "·" } else { "··" };
                    let dot_size = size * 0.6;
                    result.draws.push(Draw::Text {
                        text: text.into(),
                        x: (w - text_width(text, dot_size, false)) / 2.0,
                        baseline: y + height + size * 0.08,
                        size: dot_size,
                        italic: false,
                    });
                }
                Accent::Tilde => {
                    for i in 0..20 {
                        let x1 = w * i as f32 / 20.0;
                        let x2 = w * (i + 1) as f32 / 20.0;
                        let wave = |x: f32| {
                            y + height / 2.0 - (x / w * std::f32::consts::TAU).sin() * height * 0.4
                        };
                        result.draws.push(line(x1, wave(x1), x2, wave(x2)));
                    }
                }
                Accent::Overbrace | Accent::Underbrace => result.draws.push(Draw::Delimiter {
                    ch: if below { '⏟' } else { '⏞' },
                    rect: Rect {
                        x: 0.0,
                        y,
                        width: w,
                        height,
                    },
                    ghost: false,
                }),
            }
            result.place(body, 0.0, 0.0);
        }
        Node::Root { index, body } => {
            let body = row_layout(body, size, depth);
            let degree_width = index
                .as_ref()
                .map(|r| row_layout(r, small, depth).width * 0.6)
                .unwrap_or(0.0);
            let sign = size * 0.55;
            let x = degree_width + sign;
            let top = -body.ascent - size * 0.12;
            result.width = x + body.width + size * 0.12;
            result.ascent = -top + size * 0.08;
            result.descent = body.descent;
            for (x1, y1, x2, y2) in [
                (
                    degree_width,
                    -size * 0.15,
                    degree_width + sign * 0.22,
                    -size * 0.25,
                ),
                (
                    degree_width + sign * 0.22,
                    -size * 0.25,
                    degree_width + sign * 0.42,
                    body.descent * 0.5,
                ),
                (
                    degree_width + sign * 0.42,
                    body.descent * 0.5,
                    degree_width + sign * 0.8,
                    top,
                ),
                (degree_width + sign * 0.8, top, result.width, top),
            ] {
                result.draws.push(Draw::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width: (size * 0.045).max(1.0),
                });
            }
            if let Some(index) = index {
                let degree = row_layout(index, small, depth);
                let y = -size * 0.5;
                result.ascent = result.ascent.max(degree.ascent - y);
                result.place(degree, 0.0, y);
            }
            result.place(body, x, 0.0);
        }
        Node::Group {
            open,
            close,
            closed,
            ghost_open,
            body,
        } => {
            let body = row_layout(body, size, depth);
            let width = (size * 0.25).max((body.ascent + body.descent) * 0.10);
            let height = body.ascent + body.descent + size * 0.1;
            result.width = width * 2.0 + body.width + size * 0.08;
            result.ascent = body.ascent + size * 0.05;
            result.descent = body.descent + size * 0.05;
            let rect = Rect {
                x: 0.0,
                y: -result.ascent,
                width,
                height,
            };
            result.draws.push(Draw::Delimiter {
                ch: *open,
                rect,
                ghost: *ghost_open,
            });
            result.draws.push(Draw::Delimiter {
                ch: *close,
                rect: Rect {
                    x: result.width - width,
                    ..rect
                },
                ghost: !closed,
            });
            result.place(body, width + size * 0.04, 0.0);
        }
        Node::Large {
            placement,
            symbol,
            lower,
            upper,
        } => {
            let op = leaf(&symbol.to_string(), size * 1.45, false);
            let low = lower.as_ref().map(|r| row_layout(r, small, depth));
            let high = upper.as_ref().map(|r| row_layout(r, small, depth));
            let integral = *placement == LimitPlacement::Side
                || (*placement == LimitPlacement::Automatic && "∫∮∬∭".contains(*symbol));
            let bound_width = low
                .as_ref()
                .map_or(0.0, |b| b.width)
                .max(high.as_ref().map_or(0.0, |b| b.width));
            result.width = if integral {
                op.width + bound_width
            } else {
                op.width.max(bound_width)
            } + size * 0.1;
            result.ascent = op.ascent;
            result.descent = op.descent;
            let ox = if integral {
                0.0
            } else {
                (result.width - op.width) / 2.0
            };
            let right = op.width;
            if let Some(low) = low {
                let y = if integral {
                    size * 0.6
                } else {
                    op.descent + low.ascent + size * 0.06
                };
                let x = if integral {
                    right
                } else {
                    (result.width - low.width) / 2.0
                };
                result.descent = result.descent.max(y + low.descent);
                result.place(low, x, y);
            }
            if let Some(high) = high {
                let y = if integral {
                    -size * 0.85
                } else {
                    -op.ascent - high.descent - size * 0.08
                };
                let x = if integral {
                    right
                } else {
                    (result.width - high.width) / 2.0
                };
                result.ascent = result.ascent.max(high.ascent - y);
                result.place(high, x, y);
            }
            result.place(op, ox, 0.0);
        }
        Node::Matrix { cells, style } => {
            let boxes: Vec<Vec<_>> = cells
                .iter()
                .map(|r| {
                    r.iter()
                        .map(|c| row_layout(c, (size * 0.88).max(11.0), depth))
                        .collect()
                })
                .collect();
            let mut widths = vec![0.0_f32; cells[0].len()];
            let mut heights = vec![];
            for row in &boxes {
                let mut ascent = 0.0_f32;
                let mut descent = 0.0_f32;
                for (i, cell) in row.iter().enumerate() {
                    widths[i] = widths[i].max(cell.width);
                    ascent = ascent.max(cell.ascent);
                    descent = descent.max(cell.descent);
                }
                heights.push((ascent, descent));
            }
            let gap = size * 0.4;
            let bracket = size * 0.24;
            let (left, right) = style.delimiters();
            let left_width = if left.is_some() { bracket } else { 0.0 };
            let right_width = if right.is_some() { bracket } else { 0.0 };
            let height =
                heights.iter().map(|(a, d)| a + d).sum::<f32>() + gap * (cells.len() - 1) as f32;
            result.width = widths.iter().sum::<f32>()
                + gap * (widths.len() - 1) as f32
                + left_width
                + right_width
                + gap;
            result.ascent = height / 2.0 + size * 0.2;
            result.descent = height - result.ascent;
            let mut y = -result.ascent;
            for (r, row) in boxes.into_iter().enumerate() {
                let mut x = left_width + gap / 2.0;
                for (c, cell) in row.into_iter().enumerate() {
                    let alignment = if *style == MatrixStyle::Cases {
                        0.0
                    } else if *style == MatrixStyle::Aligned {
                        if c % 2 == 0 { 1.0 } else { 0.0 }
                    } else {
                        0.5
                    };
                    let cx = x + (widths[c] - cell.width) * alignment;
                    result.place(cell, cx, y + heights[r].0);
                    x += widths[c] + gap;
                }
                y += heights[r].0 + heights[r].1 + gap;
            }
            for (ch, x) in [(left, 0.0), (right, result.width - bracket)] {
                let Some(ch) = ch else { continue };
                result.draws.push(Draw::Delimiter {
                    ch,
                    rect: Rect {
                        x,
                        y: -result.ascent - 3.0,
                        width: bracket,
                        height: height + 6.0,
                    },
                    ghost: false,
                });
            }
        }
    }
    result
}

fn centered_stack(
    base: &Row,
    below: &Option<Row>,
    above: &Option<Row>,
    size: f32,
    depth: usize,
) -> BoxLayout {
    let base = row_layout(base, size, depth);
    let small = (size * 0.68).max(10.0);
    let below = below.as_ref().map(|row| row_layout(row, small, depth));
    let above = above.as_ref().map(|row| row_layout(row, small, depth));
    let mut result = BoxLayout {
        width: base
            .width
            .max(below.as_ref().map_or(0.0, |b| b.width))
            .max(above.as_ref().map_or(0.0, |b| b.width)),
        ascent: base.ascent,
        descent: base.descent,
        ..BoxLayout::default()
    };
    if let Some(below) = below {
        let y = base.descent + size * 0.12 + below.ascent;
        let x = (result.width - below.width) / 2.0;
        result.descent = y + below.descent;
        result.place(below, x, y);
    }
    if let Some(above) = above {
        let y = -base.ascent - size * 0.12 - above.descent;
        let x = (result.width - above.width) / 2.0;
        result.ascent = above.ascent - y;
        result.place(above, x, y);
    }
    let x = (result.width - base.width) / 2.0;
    result.place(base, x, 0.0);
    result.width += size * 0.08;
    result
}
