//! Opt-in continuous typing and field navigation. All edits stay in the editor tree.
use super::{Direction, Editor, Error, Node, Position, fields};

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
        if fields::Logarithm::ending_at(self.row(parent), index)
            .is_some_and(|log| !log.argument.nodes.is_empty())
            && let Node::Group {
                closed: true,
                ghost_open,
                ..
            } = &mut self.row_mut(parent).nodes[index]
        {
            *ghost_open = false;
        }
    }
    pub(super) fn next_field(&mut self, backward: bool) {
        let child = self.state.cursor.row;
        let Some((parent, index)) = self.state.root.parent(child) else {
            return;
        };
        let target = fields::FieldScope::containing(self.row(parent), index, child)
            .move_field(child, backward);
        self.accept_group(parent, index);
        self.move_to(target, false);
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
            let target = fields::FieldScope::containing(self.row(parent), index, child)
                .outside(direction == Direction::Left);
            self.accept_group(parent, index);
            self.move_to(target, false);
            if direction == Direction::Right {
                self.finish_root_call();
            }
        }
        Ok(())
    }

    pub(super) fn log_input(&mut self, ch: char) -> Result<bool, Error> {
        let pending = self.pending_log.take();
        if !matches!(ch, '_' | '(') || self.has_selection() {
            return Ok(false);
        }
        let cursor = self.state.cursor;
        let Some((parent, index)) = self.state.root.parent(cursor.row) else {
            return Ok(false);
        };
        let Some(log) = fields::Logarithm::starting_at(self.row(parent), index) else {
            return Ok(false);
        };
        let Some(sub) = log.base else {
            return Ok(false);
        };
        if sub.id != cursor.row || cursor.offset != sub.nodes.len() {
            return Ok(false);
        }
        let argument = log.argument.id;
        let empty = sub.nodes.is_empty();
        let simple = log.exponent.is_none();
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
