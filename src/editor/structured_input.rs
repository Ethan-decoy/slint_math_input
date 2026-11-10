//! Opt-in continuous typing and field navigation. All edits stay in the editor tree.
use super::{Direction, Editor, Error, Node, Position};

impl Editor {
    fn row_mut(&mut self, id: u64) -> &mut super::Row {
        self.state.root.find_mut(id).expect("validated row")
    }

    fn accept_group(&mut self, parent: u64, index: usize) {
        if let Node::Group {
            closed, ghost_open, ..
        } = &mut self.row_mut(parent).nodes[index]
        {
            *closed = true;
            *ghost_open = false;
        }
    }

    pub(super) fn confirm_log_argument(&mut self) {
        let Some((parent, index)) = self.state.root.parent(self.state.cursor.row) else {
            return;
        };
        if index == 0 {
            return;
        }
        let nodes = &self.row(parent).nodes;
        let is_log = match &nodes[index - 1] {
            Node::Function { name } => name == "log",
            Node::Script { base, .. } => {
                matches!(base.nodes.as_slice(), [Node::Function { name }] if name == "log")
            }
            _ => false,
        };
        if is_log
            && let Node::Group {
                closed: true,
                ghost_open,
                body,
                ..
            } = &mut self.row_mut(parent).nodes[index]
            && !body.nodes.is_empty()
        {
            *ghost_open = false;
        }
    }
    pub(super) fn next_field(&mut self, backward: bool) {
        let child = self.state.cursor.row;
        let Some((parent, index)) = self.state.root.parent(child) else {
            return;
        };
        let node = &self.row(parent).nodes[index];
        let fields = match node {
            Node::Script { sub, sup, .. } => sub.iter().chain(sup).collect::<Vec<_>>(),
            _ => node.children(),
        };
        let next = fields
            .iter()
            .position(|field| field.id == child)
            .and_then(|at| {
                if backward {
                    at.checked_sub(1)
                } else {
                    Some(at + 1)
                }
            })
            .and_then(|at| fields.get(at))
            .map(|field| Position {
                row: field.id,
                offset: if backward { field.nodes.len() } else { 0 },
            });
        let next = next.or_else(|| {
            if !backward {
                return self.logarithm_argument(parent, index, child);
            }
            // A logarithm's argument is a sibling group; return to its last
            // script field without visiting the function-name atom.
            if index > 0 && matches!(node, Node::Group { body, .. } if body.id == child) {
                let previous = &self.row(parent).nodes[index - 1];
                if let Node::Script { base, sub, sup, .. } = previous
                    && matches!(base.nodes.as_slice(), [Node::Function { name }] if name == "log")
                    && let Some(field) = sup.as_ref().or(sub.as_ref())
                {
                    return Some(Position {
                        row: field.id,
                        offset: field.nodes.len(),
                    });
                }
            }
            None
        });
        self.accept_group(parent, index);
        self.move_to(
            next.unwrap_or(Position {
                row: parent,
                offset: index + usize::from(!backward),
            }),
            false,
        );
        if !backward {
            self.finish_root_call();
        }
    }

    pub(super) fn exit_scope(&mut self, direction: Direction) -> Result<(), Error> {
        if !matches!(direction, Direction::Left | Direction::Right) {
            return Err(Error::new("Escape requires a left or right direction"));
        }
        let child = self.state.cursor.row;
        if let Some((parent, index)) = self.state.root.parent(child) {
            let log_argument = direction == Direction::Right
                && self.logarithm_argument(parent, index, child).is_some();
            self.accept_group(parent, index);
            self.move_to(
                Position {
                    row: parent,
                    offset: index
                        + usize::from(direction == Direction::Right)
                        + usize::from(log_argument),
                },
                false,
            );
            if direction == Direction::Right {
                self.finish_root_call();
            }
        }
        Ok(())
    }

    pub(super) fn log_input(&mut self, ch: char) -> Result<bool, Error> {
        let pending = self.pending_log.take();
        if self.has_selection() {
            return Ok(false);
        }
        let cursor = self.state.cursor;
        let Some((parent, index)) = self.state.root.parent(cursor.row) else {
            return Ok(false);
        };
        let nodes = &self.row(parent).nodes;
        let Node::Script {
            base,
            sub: Some(sub),
            sup,
            ..
        } = &nodes[index]
        else {
            return Ok(false);
        };
        if sub.id != cursor.row
            || cursor.offset != sub.nodes.len()
            || !matches!(base.nodes.as_slice(), [Node::Function { name }] if name == "log")
        {
            return Ok(false);
        }
        let Some(Node::Group {
            open: '(',
            close: ')',
            body,
            ..
        }) = nodes.get(index + 1)
        else {
            return Ok(false);
        };
        let argument = body.id;
        let empty = sub.nodes.is_empty();
        let simple = sup.is_none();
        if ch == '_' && empty {
            // Keep existing log_2 and log_(n+1) typing valid after automatic
            // expansion. The underscore marks an explicit base, not a nested one.
            self.pending_log = Some((cursor.row, true));
            return Ok(true);
        }
        if ch != '(' {
            return Ok(false);
        }
        if empty && !(simple && pending == Some((cursor.row, false))) {
            return Ok(false);
        }
        if empty {
            // Explicit log(x) omits the optional base; never invent its value.
            self.row_mut(parent).nodes[index] = Node::Function { name: "log".into() };
        }
        if let Node::Group { ghost_open, .. } = &mut self.row_mut(parent).nodes[index + 1] {
            *ghost_open = false;
        }
        self.move_to(
            Position {
                row: argument,
                offset: 0,
            },
            false,
        );
        Ok(true)
    }

    pub(super) fn finish_root_call(&mut self) {
        let row = self.state.cursor.row;
        let Some((parent, index)) = self.state.root.parent(row) else {
            return;
        };
        let Node::Root { body, .. } = &self.row(parent).nodes[index] else {
            return;
        };
        if body.id != row || self.state.cursor.offset != 1 {
            return;
        }
        if !matches!(
            body.nodes.as_slice(),
            [Node::Group {
                open: '(',
                close: ')',
                closed: true,
                ghost_open: false,
                ..
            }]
        ) {
            return;
        }
        let Node::Root { body, .. } = &mut self.row_mut(parent).nodes[index] else {
            unreachable!();
        };
        let Node::Group { body: grouped, .. } = body.nodes.remove(0) else {
            unreachable!();
        };
        body.nodes = grouped.nodes;
        self.move_to(
            Position {
                row: parent,
                offset: index + 1,
            },
            false,
        );
    }
}
