use crate::{
    Accent, Alphabet, Error, Layout, MatrixStyle, Rendered, catalog, latex,
    model::{MAX_BYTES, MAX_DRAFT_BYTES, Node, Row, SYMBOLS},
};
use serde::{Deserialize, Serialize};

/// A stable row identifier and a boundary between its atoms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    /// Stable identifier, preserved by drafts and undo.
    pub row: u64,
    /// Atom boundary, from zero through the row's length.
    pub offset: usize,
}
/// Direction of a cursor movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Previous visual position.
    Left,
    /// Next visual position.
    Right,
    /// Nearest position above, keeping the preferred horizontal coordinate.
    Up,
    /// Nearest position below, keeping the preferred horizontal coordinate.
    Down,
}
/// Structured insertions available to toolbars and hosts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Template {
    /// A fraction, capturing the term or selection to its left.
    Fraction,
    /// A square root.
    Root,
    /// A root with an editable degree.
    IndexedRoot,
    /// A superscript.
    Power,
    /// Square the preceding atom or selected expression, returning to its parent row.
    Square,
    /// A subscript.
    Subscript,
    /// Paired parentheses.
    Parentheses,
    /// Paired square brackets.
    Brackets,
    /// Paired braces for sets and piecewise expressions.
    Braces,
    /// Absolute value bars.
    Absolute,
    /// A summation with editable bounds.
    Sum,
    /// A product with editable bounds.
    Product,
    /// An integral with editable bounds.
    Integral,
    /// An upright limit operator with an editable condition below it.
    Limit(String),
    /// A logarithm with an editable base.
    LogBase,
    /// A differential operator with an editable denominator; order is in 1..=16.
    Derivative {
        /// Use partial differential symbols instead of ordinary differential letters.
        partial: bool,
        /// Order of the derivative, in 1..=16.
        order: u8,
    },
    /// A binomial coefficient with editable upper and lower arguments.
    Binomial,
    /// Scalable floor brackets.
    Floor,
    /// Scalable ceiling brackets.
    Ceiling,
    /// Scalable double bars.
    Norm,
    /// Scalable angle brackets.
    Angle,
    /// A scalable decoration around an editable expression.
    Accent(Accent),
    /// Apply a mathematical alphabet to an editable expression.
    Styled(Alphabet),
    /// An editable annotation centered above a base expression.
    Overset,
    /// An editable annotation centered below a base expression.
    Underset,
    /// A supported large operator, optionally with editable bounds.
    LargeOperator {
        /// Supported mathematical operator glyph.
        symbol: char,
        /// Create editable lower and upper bounds.
        bounded: bool,
    },
    /// A matrix, determinant, aligned block or piecewise expression.
    StyledMatrix {
        /// Number of rows, in 1..=16.
        rows: usize,
        /// Number of columns, in 1..=16; cases require two.
        columns: usize,
        /// Delimiters and column alignment.
        style: MatrixStyle,
    },
    /// A rectangular matrix of editable cells.
    Matrix {
        /// Number of rows, in 1..=16.
        rows: usize,
        /// Number of columns, in 1..=16.
        columns: usize,
    },
    /// Literal upright text.
    Text(String),
}
/// A serializable user operation. Failed edits leave the editor unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Type text using mathematical shortcuts. One action is one undo step.
    Insert(String),
    /// Insert one upright function name as a single editable atom.
    Function(String),
    /// Add or remove rows/columns in the containing matrix. Nonempty cells are preserved.
    ResizeMatrix {
        /// Signed change to the row count.
        row_delta: i32,
        /// Signed change to the column count.
        column_delta: i32,
    },
    /// Move the caret; the boolean extends the selection when true.
    Move(Direction, bool),
    /// Go to the row's beginning; the boolean extends selection.
    Home(bool),
    /// Go to the row's end; the boolean extends selection.
    End(bool),
    /// Exit the containing structure toward the left or right.
    Escape(Direction),
    /// Delete from the cursor to the beginning or end of its current row.
    DeleteToBoundary(Direction),
    /// Move to the document beginning, optionally extending selection.
    RootHome(bool),
    /// Move to the document end, optionally extending selection.
    RootEnd(bool),
    /// Delete to the left, entering or unwrapping structures at boundaries.
    Backspace,
    /// Delete to the right.
    Delete,
    /// Select the whole expression.
    SelectAll,
    /// Insert a structural template.
    Template(Template),
    /// Restore the preceding edit, including cursor and selection.
    Undo,
    /// Restore an undone edit.
    Redo,
    /// Empty the document, with undo support.
    Clear,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct State {
    pub root: Row,
    pub next_id: u64,
    pub cursor: Position,
    pub anchor: Option<Position>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Draft {
    version: u32,
    state: State,
}

/// An independent mathematical document and its editing history.
#[derive(Clone, Debug)]
pub struct Editor {
    pub(crate) state: State,
    undo: Vec<State>,
    redo: Vec<State>,
    preferred_x: Option<f32>,
    /// Base font size used for geometric keyboard navigation, in logical pixels.
    pub font_size: f32,
}
impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
impl Editor {
    /// Insert a built-in mathematical catalog item as one undoable action.
    pub fn insert_catalog(&mut self, id: &str) -> Result<(), Error> {
        let entry = catalog::entry(id)
            .ok_or_else(|| Error::new(format!("Unknown mathematical catalog item: {id}")))?;
        self.apply(entry.action.clone())
    }
    /// Create an empty expression.
    pub fn new() -> Self {
        Self {
            state: State {
                root: Row {
                    id: 0,
                    nodes: vec![],
                },
                next_id: 1,
                cursor: Position { row: 0, offset: 0 },
                anchor: None,
            },
            undo: vec![],
            redo: vec![],
            preferred_x: None,
            font_size: 32.0,
        }
    }
    /// Return the active cursor boundary.
    pub fn cursor(&self) -> Position {
        self.state.cursor
    }
    /// Return the selection anchor, if selection is being extended.
    pub fn anchor(&self) -> Option<Position> {
        self.state.anchor
    }
    /// Whether the anchor and caret enclose a nonempty structural selection.
    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }
    /// Whether the cursor is in the top-level expression row.
    pub fn at_root(&self) -> bool {
        self.state.cursor.row == self.state.root.id
    }
    /// Whether the last edit can be undone.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    /// Whether an edit can be redone.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    /// Strict LaTeX export: incomplete structures are reported as errors.
    pub fn latex(&self) -> Result<String, Error> {
        latex::export(&self.state.root, true)
    }
    /// LaTeX with empty arguments retained for diagnostics and clipboard exchange.
    /// Use [`Self::draft`] for lossless persistence, including ghost brackets.
    pub fn latex_preview(&self) -> String {
        latex::export(&self.state.root, false).expect("permissive export")
    }
    /// Export the selected fragment, or the entire expression if nothing is selected.
    pub fn copy_latex(&self) -> String {
        if let Some((id, start, end)) = self.selection_range() {
            let row = self.row(id);
            latex::export(
                &Row {
                    id,
                    nodes: row.nodes[start..end].to_vec(),
                },
                false,
            )
            .expect("permissive export")
        } else {
            self.latex_preview()
        }
    }
    /// Replace the document atomically with supported LaTeX. Empty arguments remain editable.
    pub fn import_latex(&mut self, source: &str) -> Result<(), Error> {
        let (root, next_id) = latex::parse(source, 0)?;
        let before = self.state.clone();
        self.state = State {
            cursor: Position {
                row: root.id,
                offset: root.nodes.len(),
            },
            root,
            next_id,
            anchor: None,
        };
        self.validate()?;
        self.record(before);
        Ok(())
    }
    /// Paste supported LaTeX or ordinary mathematical text at the selection.
    /// Parse errors preserve the existing expression and selection.
    pub fn paste(&mut self, source: &str) -> Result<(), Error> {
        if source.contains('\\')
            || source.trim_start().starts_with('$')
            || source.contains("^{")
            || source.contains("_{")
        {
            let (row, next_id) = latex::parse(source, self.state.next_id)?;
            let before = self.state.clone();
            self.state.next_id = next_id;
            self.remove_selection();
            self.splice(row.nodes);
            if let Err(error) = self.validate() {
                self.state = before;
                return Err(error);
            }
            self.record(before);
            Ok(())
        } else {
            self.apply(Action::Insert(source.into()))
        }
    }
    /// Serialize a versioned draft, preserving holes, cursor, and selection.
    /// Undo history is session-local and is deliberately not persisted.
    pub fn draft(&self) -> Result<String, Error> {
        let draft = Draft {
            version: 1,
            state: self.state.clone(),
        };
        let mut bytes = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut bytes);
        draft
            .serialize(serde_stacker::Serializer::new(&mut serializer))
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(String::from_utf8(bytes).expect("JSON serializer emits UTF-8"))
    }
    /// Load a validated draft. Unknown versions and malformed structures are rejected.
    pub fn from_draft(source: &str) -> Result<Self, Error> {
        if source.len() > MAX_DRAFT_BYTES {
            return Err(Error::new("Draft size limit exceeded"));
        }
        // Bound JSON nesting before using a growing serde stack. Valid formula
        // depth 64 can exceed serde_json's default 128 JSON-container limit.
        let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
        for (offset, byte) in source.bytes().enumerate() {
            if quoted {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quoted = false;
                }
            } else {
                match byte {
                    b'"' => quoted = true,
                    b'{' | b'[' => {
                        depth += 1;
                        if depth > 512 {
                            return Err(Error::at("Draft JSON nesting limit exceeded", offset));
                        }
                    }
                    b'}' | b']' => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
        let mut deserializer = serde_json::Deserializer::from_str(source);
        deserializer.disable_recursion_limit();
        let draft = Draft::deserialize(serde_stacker::Deserializer::new(&mut deserializer))
            .map_err(|e| Error::new(format!("Invalid draft: {e}")))?;
        deserializer
            .end()
            .map_err(|e| Error::new(format!("Invalid draft: {e}")))?;
        if draft.version != 1 {
            return Err(Error::new("Unsupported draft version"));
        }
        let editor = Self {
            state: draft.state,
            ..Self::new()
        };
        editor.validate()?;
        Ok(editor)
    }
    /// Restore a draft as an undoable edit, preserving this host's font size.
    pub fn restore_draft(&mut self, source: &str) -> Result<(), Error> {
        let incoming = Self::from_draft(source)?;
        let before = self.state.clone();
        self.state = incoming.state;
        self.record(before);
        self.preferred_x = None;
        Ok(())
    }
    /// Persist a draft with a same-directory temporary file and atomic replacement.
    pub fn save_draft_to(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        crate::storage::save(path.as_ref(), &self.draft()?)
    }
    /// Load and validate a draft from a file without opening a UI.
    pub fn load_draft_from(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        Self::from_draft(&crate::storage::load(path.as_ref())?)
    }
    /// A structural spoken summary for a host's accessibility label.
    pub fn spoken_text(&self) -> String {
        fn speak(row: &Row) -> String {
            if row.nodes.is_empty() {
                return "blank".into();
            }
            row.nodes
                .iter()
                .map(|node| match node {
                    Node::Symbol { text } => match text.as_str() {
                        "+" => "plus".into(),
                        "−" => "minus".into(),
                        "=" => "equals".into(),
                        "·" | "×" => "times".into(),
                        _ => text.clone(),
                    },
                    Node::Function { name } => name.clone(),
                    Node::Text { text } => text.clone(),
                    Node::Fraction {
                        numerator,
                        denominator,
                    } => format!(
                        "fraction numerator {} denominator {} end fraction",
                        speak(numerator),
                        speak(denominator)
                    ),
                    Node::Binomial { upper, lower } => format!(
                        "binomial upper {} lower {} end binomial",
                        speak(upper),
                        speak(lower)
                    ),
                    Node::Accent { accent, body } => {
                        format!("{accent:?} {} end accent", speak(body))
                    }
                    Node::Styled { alphabet, body } => {
                        format!("{alphabet:?} {} end alphabet", speak(body))
                    }
                    Node::Stacked { base, over, under } => format!(
                        "{} {} {}",
                        speak(base),
                        over.as_ref()
                            .map(|r| format!("above {}", speak(r)))
                            .unwrap_or_default(),
                        under
                            .as_ref()
                            .map(|r| format!("below {}", speak(r)))
                            .unwrap_or_default()
                    ),
                    Node::Root { index, body } => format!(
                        "{}root {} end root",
                        index
                            .as_ref()
                            .map(|r| format!("degree {} ", speak(r)))
                            .unwrap_or_default(),
                        speak(body)
                    ),
                    Node::Script {
                        placement: _,
                        base,
                        sub,
                        sup,
                    } => format!(
                        "{}{}{}",
                        speak(base),
                        sub.as_ref()
                            .map(|r| format!(" subscript {} end subscript", speak(r)))
                            .unwrap_or_default(),
                        sup.as_ref()
                            .map(|r| format!(" superscript {} end superscript", speak(r)))
                            .unwrap_or_default()
                    ),
                    Node::Group {
                        open,
                        close,
                        body,
                        closed,
                        ghost_open,
                    } => format!(
                        "{}open {open} {} {}close {close}",
                        if *ghost_open { "ghost " } else { "" },
                        speak(body),
                        if *closed { "" } else { "ghost " }
                    ),
                    Node::Large {
                        placement: _,
                        symbol,
                        lower,
                        upper,
                    } => format!(
                        "{symbol} lower bound {} upper bound {}",
                        lower.as_ref().map(speak).unwrap_or_else(|| "none".into()),
                        upper.as_ref().map(speak).unwrap_or_else(|| "none".into())
                    ),
                    Node::Matrix { cells, style } => format!(
                        "{style:?} matrix {} rows {} columns {} end matrix",
                        cells.len(),
                        cells[0].len(),
                        cells
                            .iter()
                            .map(|row| row
                                .iter()
                                .map(speak)
                                .collect::<Vec<_>>()
                                .join(" next cell "))
                            .collect::<Vec<_>>()
                            .join(" next row ")
                    ),
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        speak(&self.state.root)
    }
    /// Build the shared mathematical geometry in logical pixels.
    pub fn layout(&self) -> Layout {
        Layout::build(&self.state.root, self.font_size)
    }
    /// Render a transparent premultiplied RGBA image at the requested pixel density.
    pub fn render(&self, scale: f32) -> Result<Rendered, Error> {
        crate::render::render(self, scale)
    }
    /// Position or extend the caret using the same geometry that draws the expression.
    pub fn pointer(&mut self, x: f32, y: f32, extend: bool) -> Result<(), Error> {
        if !x.is_finite() || !y.is_finite() {
            return Err(Error::new("Invalid pointer coordinates"));
        }
        let position = self.layout().hit_test(x, y);
        self.set_position(position, extend)
    }
    /// Set a validated caret position. Invalid positions leave selection unchanged.
    pub fn set_position(&mut self, position: Position, extend: bool) -> Result<(), Error> {
        if !self.valid_position(position) {
            return Err(Error::new("Cursor does not belong to this document"));
        }
        self.move_to(position, extend);
        self.preferred_x = None;
        Ok(())
    }
    /// Apply one operation atomically, recording content changes for undo.
    pub fn apply(&mut self, action: Action) -> Result<(), Error> {
        if matches!(action, Action::Undo) {
            if let Some(state) = self.undo.pop() {
                self.redo.push(std::mem::replace(&mut self.state, state));
            }
            self.preferred_x = None;
            return Ok(());
        }
        if matches!(action, Action::Redo) {
            if let Some(state) = self.redo.pop() {
                self.undo.push(std::mem::replace(&mut self.state, state));
            }
            self.preferred_x = None;
            return Ok(());
        }
        let before = self.state.clone();
        if let Err(error) = self.apply_inner(action).and_then(|()| self.validate()) {
            self.state = before;
            return Err(error);
        }
        self.record(before);
        Ok(())
    }
    /// Check structural and cursor invariants; useful in operation replays.
    pub fn validate(&self) -> Result<(), Error> {
        self.state.root.validate(self.state.next_id)?;
        if !self.valid_position(self.state.cursor)
            || self.state.anchor.is_some_and(|p| !self.valid_position(p))
        {
            return Err(Error::new("Invalid draft cursor or selection"));
        }
        Ok(())
    }
    fn valid_position(&self, p: Position) -> bool {
        self.state
            .root
            .find(p.row)
            .is_some_and(|r| p.offset <= r.nodes.len())
    }
    fn record(&mut self, before: State) {
        if before.root != self.state.root {
            self.undo.push(before);
            if self.undo.len() > 128 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }
    fn row(&self, id: u64) -> &Row {
        self.state.root.find(id).expect("validated row")
    }
    fn current(&self) -> &Row {
        self.row(self.state.cursor.row)
    }
    fn current_mut(&mut self) -> &mut Row {
        self.state
            .root
            .find_mut(self.state.cursor.row)
            .expect("validated row")
    }
    fn new_row(&mut self, nodes: Vec<Node>) -> Row {
        let id = self.state.next_id;
        self.state.next_id += 1;
        Row { id, nodes }
    }
    fn splice(&mut self, nodes: Vec<Node>) {
        let at = self.state.cursor.offset;
        let length = nodes.len();
        self.current_mut().nodes.splice(at..at, nodes);
        self.state.cursor.offset += length;
        self.state.anchor = None;
    }
    pub(crate) fn selection_range(&self) -> Option<(u64, usize, usize)> {
        let a = self.state.anchor?;
        let b = self.state.cursor;
        if a == b {
            return None;
        }
        let a_path = self.state.root.ancestry(a.row);
        let b_path = self.state.root.ancestry(b.row);
        let common = *a_path.iter().find(|id| b_path.contains(id))?;
        let project = |p: Position| {
            if p.row == common {
                return (p.offset, p.offset);
            }
            let mut id = p.row;
            loop {
                let (parent, index) = self.state.root.parent(id).expect("common ancestor");
                if parent == common {
                    return (index, index + 1);
                }
                id = parent;
            }
        };
        let (a0, a1) = project(a);
        let (b0, b1) = project(b);
        Some((common, a0.min(b0), a1.max(b1)))
    }
    fn take_selection(&mut self) -> Option<Vec<Node>> {
        let (id, start, end) = self.selection_range()?;
        let nodes = self
            .state
            .root
            .find_mut(id)
            .expect("selected row")
            .nodes
            .drain(start..end)
            .collect();
        self.state.cursor = Position {
            row: id,
            offset: start,
        };
        self.state.anchor = None;
        Some(nodes)
    }
    fn remove_selection(&mut self) -> bool {
        self.take_selection().is_some()
    }
    fn move_to(&mut self, position: Position, extend: bool) {
        if extend {
            self.state.anchor.get_or_insert(self.state.cursor);
        } else {
            self.state.anchor = None;
        }
        self.state.cursor = position;
        if extend
            && let Some(anchor) = self.state.anchor
            && anchor.row != position.row
            && let Some((row, start, end)) = self.selection_range()
        {
            fn order(row: &Row, position: Position) -> Option<Vec<usize>> {
                if row.id == position.row {
                    return Some(vec![position.offset * 2]);
                }
                for (node_index, node) in row.nodes.iter().enumerate() {
                    for (child_index, child) in node.children().into_iter().enumerate() {
                        if let Some(mut suffix) = order(child, position) {
                            let mut path = vec![node_index * 2 + 1, child_index];
                            path.append(&mut suffix);
                            return Some(path);
                        }
                    }
                }
                None
            }
            let forward = order(&self.state.root, anchor) < order(&self.state.root, position);
            self.state.anchor = Some(Position {
                row,
                offset: if forward { start } else { end },
            });
            self.state.cursor = Position {
                row,
                offset: if forward { end } else { start },
            };
        }
    }
    fn apply_inner(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Function(name) => {
                if name.is_empty()
                    || name.len() > MAX_BYTES
                    || !name.chars().all(|c| c.is_ascii_alphabetic())
                {
                    return Err(Error::new("Function names must contain ASCII letters"));
                }
                self.remove_selection();
                self.splice(vec![Node::Function { name }]);
            }
            Action::ResizeMatrix {
                row_delta,
                column_delta,
            } => self.resize_matrix(row_delta, column_delta)?,
            Action::Insert(text) => {
                if text.len() > MAX_BYTES
                    || text.chars().count() > crate::model::MAX_NODES
                    || text.chars().any(char::is_control)
                {
                    return Err(Error::new("Input must be single-line mathematical text"));
                }
                for ch in text.chars() {
                    self.type_char(ch)?;
                }
                self.preferred_x = None;
            }
            Action::Template(template) => {
                self.template(template)?;
                self.preferred_x = None;
            }
            Action::Move(direction, extend) => self.move_direction(direction, extend),
            Action::Home(extend) => self.move_to(
                Position {
                    row: self.state.cursor.row,
                    offset: 0,
                },
                extend,
            ),
            Action::End(extend) => self.move_to(
                Position {
                    row: self.state.cursor.row,
                    offset: self.current().nodes.len(),
                },
                extend,
            ),
            Action::RootHome(extend) => self.move_to(
                Position {
                    row: self.state.root.id,
                    offset: 0,
                },
                extend,
            ),
            Action::RootEnd(extend) => self.move_to(
                Position {
                    row: self.state.root.id,
                    offset: self.state.root.nodes.len(),
                },
                extend,
            ),
            Action::Escape(direction) => {
                if matches!(direction, Direction::Up | Direction::Down) {
                    return Err(Error::new("Escape requires a left or right direction"));
                }
                if let Some((row, index)) = self.state.root.parent(self.state.cursor.row) {
                    self.move_to(
                        Position {
                            row,
                            offset: index + usize::from(direction == Direction::Right),
                        },
                        false,
                    );
                }
            }
            Action::DeleteToBoundary(direction) => {
                if matches!(direction, Direction::Up | Direction::Down) {
                    return Err(Error::new(
                        "Boundary deletion requires a left or right direction",
                    ));
                }
                if self.selection_range().is_none() {
                    let forward = direction == Direction::Right;
                    let target = if forward {
                        self.current().nodes.len()
                    } else {
                        0
                    };
                    if target == self.state.cursor.offset {
                        self.delete(forward);
                        return Ok(());
                    }
                    self.state.anchor = Some(self.state.cursor);
                    self.state.cursor.offset = target;
                }
                self.remove_selection();
            }
            Action::SelectAll => {
                self.state.anchor = Some(Position {
                    row: self.state.root.id,
                    offset: 0,
                });
                self.state.cursor = Position {
                    row: self.state.root.id,
                    offset: self.state.root.nodes.len(),
                };
            }
            Action::Backspace => self.delete(false),
            Action::Delete => self.delete(true),
            Action::Clear => {
                self.state.root.nodes.clear();
                self.state.cursor = Position {
                    row: self.state.root.id,
                    offset: 0,
                };
                self.state.anchor = None;
            }
            Action::Undo | Action::Redo => unreachable!(),
        }
        Ok(())
    }
    fn type_char(&mut self, ch: char) -> Result<(), Error> {
        if matches!(ch, '^' | '_')
            && self.selection_range().is_none()
            && self.state.cursor.offset == self.current().nodes.len()
            && let Some((parent, index)) = self.state.root.parent(self.state.cursor.row)
            && let Node::Script { sub, sup, .. } = &self.row(parent).nodes[index]
            && ((ch == '^'
                && sub
                    .as_ref()
                    .is_some_and(|row| row.id == self.state.cursor.row))
                || (ch == '_'
                    && sup
                        .as_ref()
                        .is_some_and(|row| row.id == self.state.cursor.row)))
        {
            self.state.cursor = Position {
                row: parent,
                offset: index + 1,
            };
        }
        if "+-=<>*".contains(ch)
            && self.selection_range().is_none()
            && self.state.cursor.offset > 0
            && self.state.cursor.offset == self.current().nodes.len()
            && let Some((parent, index)) = self.state.root.parent(self.state.cursor.row)
            && let Node::Script {
                placement: _,
                base,
                sub,
                sup,
            } = &self.row(parent).nodes[index]
            && !base.centers_limits()
            && (sub.as_ref().is_some_and(|r| r.id == self.state.cursor.row)
                || sup.as_ref().is_some_and(|r| r.id == self.state.cursor.row))
        {
            self.state.cursor = Position {
                row: parent,
                offset: index + 1,
            };
        }
        match ch {
            '/' => return self.template(Template::Fraction),
            '^' => return self.template(Template::Power),
            '_' => return self.template(Template::Subscript),
            '(' | '[' | '{' | '⌊' | '⌈' | '⟨' => {
                if self.fill_opening(ch) {
                    return Ok(());
                }
                return self.template(match ch {
                    '(' => Template::Parentheses,
                    '[' => Template::Brackets,
                    '⌊' => Template::Floor,
                    '⌈' => Template::Ceiling,
                    '⟨' => Template::Angle,
                    _ => Template::Braces,
                });
            }
            '|' => {
                if self.close_group('|') {
                    return Ok(());
                }
                return self.template(Template::Absolute);
            }
            '‖' => {
                if self.close_group(ch) {
                    return Ok(());
                }
                return self.template(Template::Norm);
            }
            ')' | ']' | '}' | '⌋' | '⌉' | '⟩' => {
                if self.selection_range().is_none() && self.close_group(ch) {
                    return Ok(());
                }
                self.open_from_right(ch);
                return Ok(());
            }
            ' ' => {
                self.move_direction(Direction::Right, false);
                return Ok(());
            }
            '√' => return self.template(Template::Root),
            _ => {}
        }
        self.remove_selection();
        let value = match ch {
            '*' => '·',
            '-' => '−',
            '\'' => '′',
            _ => ch,
        };
        let at = self.state.cursor.offset;
        if value == '>' && at > 0 && self.current().nodes[at - 1].text() == Some("−") {
            self.current_mut().nodes[at - 1] = Node::symbol("→");
            return Ok(());
        }
        if value == '=' && at > 0 {
            let prior = self.current().nodes[at - 1].text().unwrap_or("");
            let replacement = match prior {
                "<" => Some("≤"),
                ">" => Some("≥"),
                "!" => Some("≠"),
                _ => None,
            };
            if let Some(text) = replacement {
                self.current_mut().nodes[at - 1] = Node::symbol(text);
                return Ok(());
            }
        }
        self.splice(vec![Node::symbol(value.to_string())]);
        let explicit_alphabet =
            self.state
                .root
                .parent(self.state.cursor.row)
                .is_some_and(|(parent, index)| {
                    matches!(self.row(parent).nodes[index], Node::Styled { .. })
                });
        if ch.is_ascii_alphabetic() && !explicit_alphabet {
            self.auto_command()?;
        }
        Ok(())
    }
    fn auto_command(&mut self) -> Result<(), Error> {
        fn letters(node: &Node) -> Option<&str> {
            match node {
                Node::Function { name } => Some(name),
                Node::Symbol { text } if text.chars().all(|c| c.is_ascii_alphabetic()) => {
                    Some(text)
                }
                _ => None,
            }
        }
        let end = self.state.cursor.offset;
        let mut start = end;
        while start > 0 && letters(&self.current().nodes[start - 1]).is_some() {
            start -= 1;
        }
        let mut word = String::new();
        let mut boundaries = vec![];
        for index in start..end {
            boundaries.push((word.len(), index));
            word.push_str(letters(&self.current().nodes[index]).expect("letter token"));
        }
        let boundary = |name: &str| {
            word.strip_suffix(name).and_then(|prefix| {
                boundaries
                    .iter()
                    .find(|(offset, _)| *offset == prefix.len())
                    .map(|(_, index)| *index)
            })
        };
        let symbol = SYMBOLS
            .iter()
            .filter(|(name, _)| {
                (name.len() >= 3 || matches!(*name, "pi" | "mu" | "nu" | "xi"))
                    && boundary(name).is_some()
            })
            .max_by_key(|(name, _)| name.len());
        let structural = [
            "sqrt", "nthroot", "frac", "binom", "ddx", "pdiff", "hat", "bar", "vec",
        ]
        .into_iter()
        .chain(crate::model::LARGE_OPERATORS.iter().map(|(name, _)| *name))
        .filter(|name| boundary(name).is_some())
        .max_by_key(|name| name.len());
        let function = catalog::function_names()
            .filter(|name| boundary(name).is_some())
            .max_by_key(|name| name.len());
        let symbol_len = symbol.map_or(0, |(name, _)| name.len());
        let structural_len = structural.map_or(0, str::len);
        let function_len = function.map_or(0, str::len);
        let Some(name) = symbol
            .map(|(name, _)| *name)
            .filter(|_| symbol_len >= structural_len && symbol_len >= function_len)
            .or_else(|| structural.filter(|_| structural_len >= function_len))
            .or(function)
        else {
            return Ok(());
        };
        let mut from = boundary(name).expect("matched token boundary");
        if from > 0 && self.current().nodes[from - 1].text() == Some("\\") {
            from -= 1;
        }
        self.current_mut().nodes.drain(from..end);
        self.state.cursor.offset = from;
        if symbol.is_some_and(|(command, _)| *command == name) {
            self.splice(vec![Node::symbol(symbol.expect("matched symbol").1)]);
        } else if structural == Some(name) {
            let template = match name {
                "sqrt" => Template::Root,
                "nthroot" => Template::IndexedRoot,
                "frac" => Template::Fraction,
                "binom" => Template::Binomial,
                "ddx" => Template::Derivative {
                    partial: false,
                    order: 1,
                },
                "pdiff" => Template::Derivative {
                    partial: true,
                    order: 1,
                },
                "hat" => Template::Accent(Accent::Hat),
                "bar" => Template::Accent(Accent::Bar),
                "vec" => Template::Accent(Accent::Vector),
                "sum" => Template::Sum,
                "prod" => Template::Product,
                "int" => Template::Integral,
                _ => Template::LargeOperator {
                    symbol: crate::model::LARGE_OPERATORS
                        .iter()
                        .find(|(command, _)| *command == name)
                        .expect("matched operator")
                        .1,
                    bounded: true,
                },
            };
            self.template(template)?;
        } else {
            self.splice(vec![Node::Function { name: name.into() }]);
        }
        Ok(())
    }
    fn close_group(&mut self, close: char) -> bool {
        if let Some((parent, index)) = self.state.root.parent(self.state.cursor.row)
            && let Node::Group {
                close: expected,
                closed,
                body,
                ..
            } = &mut self.state.root.find_mut(parent).expect("parent").nodes[index]
            && *expected == close
            && body.id == self.state.cursor.row
        {
            *closed = true;
            let tail = body.nodes.split_off(self.state.cursor.offset);
            self.state
                .root
                .find_mut(parent)
                .expect("parent")
                .nodes
                .splice(index + 1..index + 1, tail);
            self.state.cursor = Position {
                row: parent,
                offset: index + 1,
            };
            self.state.anchor = None;
            return true;
        }
        false
    }
    fn fill_opening(&mut self, open: char) -> bool {
        let cursor = self.state.cursor;
        if let Some(Node::Group {
            open: expected,
            ghost_open,
            body,
            ..
        }) = self.current_mut().nodes.get_mut(cursor.offset)
            && *ghost_open
            && *expected == open
        {
            *ghost_open = false;
            self.state.cursor = Position {
                row: body.id,
                offset: 0,
            };
            return true;
        }
        if let Some((parent, index)) = self.state.root.parent(cursor.row)
            && let Node::Group {
                open: expected,
                ghost_open,
                body,
                ..
            } = &mut self.state.root.find_mut(parent).expect("parent").nodes[index]
            && *ghost_open
            && *expected == open
        {
            *ghost_open = false;
            let prefix: Vec<_> = body.nodes.drain(..cursor.offset).collect();
            let body_id = body.id;
            self.state
                .root
                .find_mut(parent)
                .expect("parent")
                .nodes
                .splice(index..index, prefix);
            self.state.cursor = Position {
                row: body_id,
                offset: 0,
            };
            return true;
        }
        false
    }
    fn open_from_right(&mut self, close: char) {
        let selected = self.take_selection();
        let ghost_open = selected.is_none();
        let contents = if let Some(selected) = selected {
            selected
        } else {
            let at = self.state.cursor.offset;
            self.state.cursor.offset = 0;
            self.current_mut().nodes.drain(..at).collect()
        };
        let body = self.new_row(contents);
        self.splice(vec![Node::Group {
            open: match close {
                ']' => '[',
                '}' => '{',
                '⌋' => '⌊',
                '⌉' => '⌈',
                '⟩' => '⟨',
                _ => '(',
            },
            close,
            closed: true,
            ghost_open,
            body,
        }]);
    }
    fn template(&mut self, template: Template) -> Result<(), Error> {
        let selection = self.take_selection();
        let has_selection = selection.is_some();
        let selected = selection.unwrap_or_default();
        let mut trailing = vec![];
        let (node, target) = match template {
            Template::Limit(ref name) => {
                if !matches!(
                    name.as_str(),
                    "lim" | "limsup" | "liminf" | "sup" | "inf" | "max" | "min"
                ) {
                    return Err(Error::new("Unsupported limit operator"));
                }
                let base = self.new_row(vec![Node::Function { name: name.clone() }]);
                let sub = self.new_row(vec![]);
                let target = sub.id;
                trailing = selected;
                (
                    Node::Script {
                        placement: Default::default(),
                        base,
                        sub: Some(sub),
                        sup: None,
                    },
                    target,
                )
            }
            Template::LogBase => {
                let base = self.new_row(vec![Node::Function { name: "log".into() }]);
                let sub = self.new_row(vec![]);
                let target = sub.id;
                trailing = selected;
                (
                    Node::Script {
                        placement: Default::default(),
                        base,
                        sub: Some(sub),
                        sup: None,
                    },
                    target,
                )
            }
            Template::Derivative { partial, order } => {
                if !(1..=16).contains(&order) {
                    return Err(Error::new("Derivative order must be in 1..=16"));
                }
                let differential = if partial { "∂" } else { "d" };
                let numerator_nodes = if order == 1 {
                    vec![Node::symbol(differential)]
                } else {
                    let base = self.new_row(vec![Node::symbol(differential)]);
                    let sup = self.new_row(
                        order
                            .to_string()
                            .chars()
                            .map(|c| Node::symbol(c.to_string()))
                            .collect(),
                    );
                    vec![Node::Script {
                        placement: Default::default(),
                        base,
                        sub: None,
                        sup: Some(sup),
                    }]
                };
                let numerator = self.new_row(numerator_nodes);
                let mut denominator = vec![Node::symbol(differential)];
                if order == 1 {
                    denominator.push(Node::symbol("x"));
                } else {
                    let base = self.new_row(vec![Node::symbol("x")]);
                    let sup = self.new_row(
                        order
                            .to_string()
                            .chars()
                            .map(|c| Node::symbol(c.to_string()))
                            .collect(),
                    );
                    denominator.push(Node::Script {
                        placement: Default::default(),
                        base,
                        sub: None,
                        sup: Some(sup),
                    });
                }
                let denominator = self.new_row(denominator);
                self.splice(vec![Node::Fraction {
                    numerator,
                    denominator,
                }]);
                self.splice(selected);
                return Ok(());
            }
            Template::Binomial => {
                let upper = self.new_row(selected);
                let lower = self.new_row(vec![]);
                let target = if upper.nodes.is_empty() {
                    upper.id
                } else {
                    lower.id
                };
                (Node::Binomial { upper, lower }, target)
            }
            Template::Accent(accent) => {
                let body = self.new_row(selected);
                let target = body.id;
                (Node::Accent { accent, body }, target)
            }
            Template::Styled(alphabet) => {
                let body = self.new_row(selected);
                let target = body.id;
                (Node::Styled { alphabet, body }, target)
            }
            Template::Overset | Template::Underset => {
                let base = self.new_row(selected);
                let annotation = self.new_row(vec![]);
                let target = annotation.id;
                let over = matches!(template, Template::Overset);
                (
                    Node::Stacked {
                        base,
                        over: over.then(|| annotation.clone()),
                        under: (!over).then_some(annotation),
                    },
                    target,
                )
            }
            Template::Fraction => {
                let contents = if has_selection {
                    selected
                } else {
                    let end = self.state.cursor.offset;
                    let mut start = end;
                    while start > 0 && !self.current().nodes[start - 1].is_boundary() {
                        start -= 1;
                    }
                    self.state.cursor.offset = start;
                    self.current_mut().nodes.drain(start..end).collect()
                };
                let numerator = self.new_row(contents);
                let denominator = self.new_row(vec![]);
                let target = if numerator.nodes.is_empty() {
                    numerator.id
                } else {
                    denominator.id
                };
                (
                    Node::Fraction {
                        numerator,
                        denominator,
                    },
                    target,
                )
            }
            Template::Square => {
                let at = self.state.cursor.offset;
                let contents = if has_selection {
                    selected
                } else if at > 0 {
                    self.state.cursor.offset -= 1;
                    vec![self.current_mut().nodes.remove(at - 1)]
                } else {
                    return Ok(());
                };
                let base = self.new_row(contents);
                let sup = self.new_row(vec![Node::symbol("2")]);
                self.splice(vec![Node::Script {
                    placement: Default::default(),
                    base,
                    sub: None,
                    sup: Some(sup),
                }]);
                return Ok(());
            }
            Template::Power | Template::Subscript => {
                let upper = matches!(template, Template::Power);
                let at = self.state.cursor.offset;
                if at == 0 && !has_selection {
                    return Ok(());
                }
                if at > 0 {
                    let new = self.new_row(vec![]);
                    let candidate = &mut self.current_mut().nodes[at - 1];
                    if let Node::Script { sup, sub, .. } = candidate {
                        let row = if upper { sup } else { sub };
                        let active = row.get_or_insert(new);
                        if has_selection {
                            active.nodes.extend(selected);
                        }
                        self.state.cursor = Position {
                            row: active.id,
                            offset: active.nodes.len(),
                        };
                        return Ok(());
                    }
                    if let Node::Large {
                        upper: high,
                        lower: low,
                        ..
                    } = candidate
                    {
                        let active = if upper { high } else { low };
                        let row = active.get_or_insert(new);
                        if has_selection {
                            row.nodes.extend(selected);
                        }
                        self.state.cursor = Position {
                            row: row.id,
                            offset: row.nodes.len(),
                        };
                        return Ok(());
                    }
                }
                let base_nodes = if at > 0 {
                    vec![self.current_mut().nodes.remove(at - 1)]
                } else {
                    vec![]
                };
                if at > 0 {
                    self.state.cursor.offset -= 1;
                }
                let base = self.new_row(base_nodes);
                let power = self.new_row(selected);
                let target = power.id;
                let node = if upper {
                    Node::Script {
                        placement: Default::default(),
                        base,
                        sub: None,
                        sup: Some(power),
                    }
                } else {
                    Node::Script {
                        placement: Default::default(),
                        base,
                        sub: Some(power),
                        sup: None,
                    }
                };
                (node, target)
            }
            Template::Root | Template::IndexedRoot => {
                let body = self.new_row(selected);
                let index = matches!(template, Template::IndexedRoot).then(|| self.new_row(vec![]));
                let target = index.as_ref().unwrap_or(&body).id;
                (Node::Root { index, body }, target)
            }
            Template::Parentheses
            | Template::Brackets
            | Template::Braces
            | Template::Absolute
            | Template::Floor
            | Template::Ceiling
            | Template::Norm
            | Template::Angle => {
                let (open, close) = match template {
                    Template::Brackets => ('[', ']'),
                    Template::Braces => ('{', '}'),
                    Template::Absolute => ('|', '|'),
                    Template::Floor => ('⌊', '⌋'),
                    Template::Ceiling => ('⌈', '⌉'),
                    Template::Norm => ('‖', '‖'),
                    Template::Angle => ('⟨', '⟩'),
                    _ => ('(', ')'),
                };
                let contents = if has_selection {
                    selected
                } else {
                    let at = self.state.cursor.offset;
                    self.current_mut().nodes.split_off(at)
                };
                let body = self.new_row(contents);
                let target = body.id;
                (
                    Node::Group {
                        open,
                        close,
                        body,
                        closed: has_selection,
                        ghost_open: false,
                    },
                    target,
                )
            }
            Template::Sum
            | Template::Product
            | Template::Integral
            | Template::LargeOperator { .. } => {
                let symbol = match template {
                    Template::Sum => '∑',
                    Template::Product => '∏',
                    Template::LargeOperator { symbol, .. } => symbol,
                    _ => '∫',
                };
                let lower = self.new_row(if symbol == '∫' {
                    vec![]
                } else {
                    vec![Node::symbol("n"), Node::symbol("=")]
                });
                let upper = self.new_row(vec![]);
                let target = lower.id;
                trailing = selected;
                let bounded = !matches!(template, Template::LargeOperator { bounded: false, .. });
                if !bounded {
                    self.splice(vec![Node::Large {
                        placement: Default::default(),
                        symbol,
                        lower: None,
                        upper: None,
                    }]);
                    self.splice(trailing);
                    return Ok(());
                }
                (
                    Node::Large {
                        placement: Default::default(),
                        symbol,
                        lower: Some(lower),
                        upper: Some(upper),
                    },
                    target,
                )
            }
            Template::Matrix { rows, columns } | Template::StyledMatrix { rows, columns, .. } => {
                if rows == 0 || columns == 0 || rows > 16 || columns > 16 {
                    return Err(Error::new("Matrix dimensions must be in 1..=16"));
                }
                let style = if let Template::StyledMatrix { style, .. } = template {
                    style
                } else {
                    MatrixStyle::Brackets
                };
                if style == MatrixStyle::Cases && columns != 2 {
                    return Err(Error::new("Cases need two columns"));
                }
                let mut cells: Vec<Vec<Row>> = (0..rows)
                    .map(|_| (0..columns).map(|_| self.new_row(vec![])).collect())
                    .collect();
                let target = cells[0][0].id;
                cells[0][0].nodes = selected;
                (Node::Matrix { cells, style }, target)
            }
            Template::Text(text) => {
                self.splice(vec![Node::Text { text }]);
                return Ok(());
            }
        };
        self.splice(vec![node]);
        let at = self.state.cursor.offset;
        self.current_mut().nodes.splice(at..at, trailing);
        let offset = if matches!(
            template,
            Template::Parentheses
                | Template::Brackets
                | Template::Braces
                | Template::Absolute
                | Template::Floor
                | Template::Ceiling
                | Template::Norm
                | Template::Angle
        ) {
            0
        } else {
            self.row(target).nodes.len()
        };
        self.state.cursor = Position {
            row: target,
            offset,
        };
        self.state.anchor = None;
        Ok(())
    }
    fn resize_matrix(&mut self, row_delta: i32, column_delta: i32) -> Result<(), Error> {
        let mut child = self.state.cursor.row;
        while let Some((parent, index)) = self.state.root.parent(child) {
            if let Node::Matrix { cells, style } = &self.row(parent).nodes[index] {
                let rows = i32::try_from(cells.len())
                    .unwrap()
                    .checked_add(row_delta)
                    .ok_or_else(|| Error::new("Matrix dimension overflow"))?;
                let columns = i32::try_from(cells[0].len())
                    .unwrap()
                    .checked_add(column_delta)
                    .ok_or_else(|| Error::new("Matrix dimension overflow"))?;
                if !(1..=16).contains(&rows) || !(1..=16).contains(&columns) {
                    return Err(Error::new("Matrix dimensions must be in 1..=16"));
                }
                if *style == MatrixStyle::Cases && columns != 2 {
                    return Err(Error::new("Cases need two columns"));
                }
                let (rows, columns) = (rows as usize, columns as usize);
                if cells.iter().enumerate().any(|(r, row)| {
                    row.iter()
                        .enumerate()
                        .any(|(c, cell)| (r >= rows || c >= columns) && !cell.nodes.is_empty())
                }) {
                    return Err(Error::new(
                        "Remove cell contents before shrinking the matrix",
                    ));
                }
                let mut cells = cells.clone();
                let style = *style;
                cells.truncate(rows);
                for row in &mut cells {
                    row.truncate(columns);
                    while row.len() < columns {
                        row.push(self.new_row(vec![]));
                    }
                }
                while cells.len() < rows {
                    cells.push((0..columns).map(|_| self.new_row(vec![])).collect());
                }
                let cursor_exists = cells
                    .iter()
                    .flatten()
                    .any(|cell| cell.find(self.state.cursor.row).is_some());
                let fallback = cells[rows - 1][columns - 1].id;
                self.state
                    .root
                    .find_mut(parent)
                    .expect("matrix parent")
                    .nodes[index] = Node::Matrix { cells, style };
                if !cursor_exists {
                    self.state.cursor = Position {
                        row: fallback,
                        offset: 0,
                    };
                }
                self.state.anchor = None;
                return Ok(());
            }
            child = parent;
        }
        Err(Error::new(
            "Place the cursor in a matrix or piecewise expression first",
        ))
    }
    fn move_direction(&mut self, direction: Direction, extend: bool) {
        if matches!(direction, Direction::Up | Direction::Down) {
            let layout = self.layout();
            let x = self
                .preferred_x
                .unwrap_or_else(|| layout.caret(self.state.cursor).map_or(0.0, |c| c.rect.x));
            let down = direction == Direction::Down;
            let mut child_id = self.state.cursor.row;
            let mut target_row = None;
            while let Some((parent, index)) = self.state.root.parent(child_id) {
                target_row = match &self.row(parent).nodes[index] {
                    Node::Fraction {
                        numerator,
                        denominator,
                    }
                    | Node::Binomial {
                        upper: numerator,
                        lower: denominator,
                    } => {
                        if down && child_id == numerator.id {
                            Some(denominator.id)
                        } else if !down && child_id == denominator.id {
                            Some(numerator.id)
                        } else {
                            None
                        }
                    }
                    Node::Script {
                        placement: _,
                        base,
                        sub,
                        sup,
                    }
                    | Node::Stacked {
                        base,
                        under: sub,
                        over: sup,
                    } => {
                        if child_id == base.id {
                            if down {
                                sub.as_ref().map(|r| r.id)
                            } else {
                                sup.as_ref().map(|r| r.id)
                            }
                        } else if (down && sup.as_ref().is_some_and(|r| r.id == child_id))
                            || (!down && sub.as_ref().is_some_and(|r| r.id == child_id))
                        {
                            Some(base.id)
                        } else {
                            None
                        }
                    }
                    Node::Root {
                        index: Some(index),
                        body,
                    } => {
                        if down && child_id == index.id {
                            Some(body.id)
                        } else if !down && child_id == body.id {
                            Some(index.id)
                        } else {
                            None
                        }
                    }
                    Node::Large { lower, upper, .. } => {
                        if down && upper.as_ref().is_some_and(|r| child_id == r.id) {
                            lower.as_ref().map(|r| r.id)
                        } else if !down && lower.as_ref().is_some_and(|r| child_id == r.id) {
                            upper.as_ref().map(|r| r.id)
                        } else {
                            None
                        }
                    }
                    Node::Matrix { cells, .. } => {
                        let cell = cells.iter().enumerate().find_map(|(r, row)| {
                            row.iter().position(|c| c.id == child_id).map(|c| (r, c))
                        });
                        cell.and_then(|(r, c)| {
                            let next_row = if down {
                                cells.get(r + 1)
                            } else {
                                r.checked_sub(1).and_then(|r| cells.get(r))
                            };
                            next_row.map(|row| row[c].id)
                        })
                    }
                    _ => None,
                };
                if target_row.is_some() {
                    break;
                }
                child_id = parent;
            }
            let position = target_row
                .and_then(|row| layout.nearest_in_row(row, x))
                .or_else(|| layout.vertical(self.state.cursor, down, x));
            if let Some(position) = position {
                self.move_to(position, extend);
            }
            self.preferred_x = Some(x);
            return;
        }
        self.preferred_x = None;
        let right = direction == Direction::Right;
        if !extend && let Some((row, start, end)) = self.selection_range() {
            self.move_to(
                Position {
                    row,
                    offset: if right { end } else { start },
                },
                false,
            );
            return;
        }
        let cursor = self.state.cursor;
        let index = if right {
            cursor.offset
        } else {
            cursor.offset.saturating_sub(1)
        };
        if (right && cursor.offset < self.current().nodes.len()) || (!right && cursor.offset > 0) {
            let node = &self.current().nodes[index];
            let entry = if let Node::Fraction {
                numerator,
                denominator,
            }
            | Node::Binomial {
                upper: numerator,
                lower: denominator,
            } = node
            {
                Some(if right { numerator } else { denominator })
            } else if let Node::Matrix { cells, .. } = node {
                if right {
                    cells.first().and_then(|r| r.first())
                } else {
                    cells.last().and_then(|r| r.last())
                }
            } else if let Node::Script { base, .. } | Node::Stacked { base, .. } = node {
                if right { Some(base) } else { node.entry() }
            } else {
                node.entry()
            };
            let target = if let Some(row) = entry.filter(|_| !extend) {
                Position {
                    row: row.id,
                    offset: if right { 0 } else { row.nodes.len() },
                }
            } else {
                Position {
                    row: cursor.row,
                    offset: if right {
                        cursor.offset + 1
                    } else {
                        cursor.offset - 1
                    },
                }
            };
            self.move_to(target, extend);
        } else if let Some((parent, node_index)) = self.state.root.parent(cursor.row) {
            let node = &self.row(parent).nodes[node_index];
            let mut target = Position {
                row: parent,
                offset: node_index + usize::from(right),
            };
            if !extend
                && let Node::Fraction {
                    numerator,
                    denominator,
                }
                | Node::Binomial {
                    upper: numerator,
                    lower: denominator,
                } = node
            {
                if right && cursor.row == numerator.id {
                    target = Position {
                        row: denominator.id,
                        offset: 0,
                    };
                } else if !right && cursor.row == denominator.id {
                    target = Position {
                        row: numerator.id,
                        offset: numerator.nodes.len(),
                    };
                }
            }
            if !extend && let Node::Matrix { cells, .. } = node {
                let cells: Vec<_> = cells.iter().flatten().collect();
                if let Some(index) = cells.iter().position(|r| r.id == cursor.row) {
                    let adjacent = if right {
                        cells.get(index + 1)
                    } else {
                        index.checked_sub(1).and_then(|i| cells.get(i))
                    };
                    if let Some(row) = adjacent {
                        target = Position {
                            row: row.id,
                            offset: if right { 0 } else { row.nodes.len() },
                        };
                    }
                }
            }
            if !extend
                && let Node::Script {
                    placement: _,
                    base,
                    sup,
                    sub,
                }
                | Node::Stacked {
                    base,
                    over: sup,
                    under: sub,
                } = node
            {
                let script = sup.as_ref().or(sub.as_ref());
                if base.id == cursor.row && right {
                    if let Some(script) = script {
                        target = Position {
                            row: script.id,
                            offset: 0,
                        };
                    }
                } else if base.id != cursor.row && !right {
                    target = Position {
                        row: base.id,
                        offset: base.nodes.len(),
                    };
                }
            }
            self.move_to(target, extend);
        }
    }
    fn delete(&mut self, forward: bool) {
        if self.remove_selection() {
            return;
        }
        let cursor = self.state.cursor;
        let len = self.current().nodes.len();
        if (!forward && cursor.offset > 0) || (forward && cursor.offset < len) {
            let index = if forward {
                cursor.offset
            } else {
                cursor.offset - 1
            };
            if !forward {
                let replacement = match self.current().nodes[index].text() {
                    Some("≤") => Some("<"),
                    Some("≥") => Some(">"),
                    Some("→") => Some("−"),
                    _ => None,
                };
                if let Some(text) = replacement {
                    self.current_mut().nodes[index] = Node::symbol(text);
                    return;
                }
            }
            if let Node::Group {
                closed,
                ghost_open,
                body,
                ..
            } = &mut self.current_mut().nodes[index]
            {
                if (forward && !*closed) || (!forward && *ghost_open) {
                    let node = self.current_mut().nodes.remove(index);
                    let contents = node.into_contents();
                    let length = contents.len();
                    self.current_mut().nodes.splice(index..index, contents);
                    self.state.cursor.offset = index + if forward { 0 } else { length };
                    return;
                }
                if forward {
                    *ghost_open = true;
                } else {
                    *closed = false;
                }
                self.state.cursor = Position {
                    row: body.id,
                    offset: if forward { 0 } else { body.nodes.len() },
                };
                return;
            }
            if forward && matches!(&self.current().nodes[index], Node::Root { index: None, .. }) {
                let node = self.current_mut().nodes.remove(index);
                let contents = node.into_contents();
                self.current_mut().nodes.splice(index..index, contents);
                return;
            }
            let node = &self.current().nodes[index];
            let children = node.children();
            if let Some(row) = if forward {
                children.first()
            } else {
                children.last()
            } && !node.children().iter().all(|r| r.nodes.is_empty())
            {
                self.state.cursor = Position {
                    row: row.id,
                    offset: if forward { 0 } else { row.nodes.len() },
                };
                return;
            }
            self.current_mut().nodes.remove(index);
            self.state.cursor.offset = index;
        } else if let Some((parent, index)) = self.state.root.parent(cursor.row) {
            let node = &self.row(parent).nodes[index];
            let mut offset = 0;
            for child in node.children() {
                if child.id == cursor.row {
                    if forward {
                        offset += child.nodes.len();
                    }
                    break;
                }
                offset += child.nodes.len();
            }
            let node = self
                .state
                .root
                .find_mut(parent)
                .expect("parent row")
                .nodes
                .remove(index);
            let contents = node.into_contents();
            self.state
                .root
                .find_mut(parent)
                .expect("parent row")
                .nodes
                .splice(index..index, contents);
            self.state.cursor = Position {
                row: parent,
                offset: index + offset,
            };
        }
        self.preferred_x = None;
    }
}
