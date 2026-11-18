use super::{Node, Position, Row};

/// A log and its adjacent argument are two nodes in the document and one set
/// of editable fields. Resolve them in their own row, including in old drafts.
pub(super) struct Logarithm<'a> {
    pub(super) base: Option<&'a Row>,
    pub(super) exponent: Option<&'a Row>,
    pub(super) argument: &'a Row,
}

impl<'a> Logarithm<'a> {
    pub(super) fn starting_at(row: &'a Row, index: usize) -> Option<Self> {
        let [
            operator,
            Node::Group {
                open: '(',
                close: ')',
                body,
                ..
            },
            ..,
        ] = row.nodes.get(index..)?
        else {
            return None;
        };
        let (name, base, exponent) = match operator {
            Node::Script { base, sub, sup, .. } => {
                (base.nodes.as_slice(), sub.as_ref(), sup.as_ref())
            }
            atom => (std::slice::from_ref(atom), None, None),
        };
        matches!(name, [Node::Function { name }] if name == "log").then_some(Self {
            base,
            exponent,
            argument: body,
        })
    }

    pub(super) fn ending_at(row: &'a Row, index: usize) -> Option<Self> {
        Self::starting_at(row, index.checked_sub(1)?)
    }

    pub(super) fn has_script(&self, child: u64) -> bool {
        self.base
            .into_iter()
            .chain(self.exponent)
            .any(|field| field.id == child)
    }
}

/// A field command can traverse a single node or a scripted log and its argument.
pub(super) struct FieldScope<'a> {
    parent: &'a Row,
    start: usize,
    argument: Option<&'a Row>,
}

impl<'a> FieldScope<'a> {
    pub(super) fn containing(parent: &'a Row, index: usize, child: u64) -> Self {
        let start = match parent.nodes[index] {
            Node::Group { .. } => index.checked_sub(1),
            Node::Script { .. } => Some(index),
            _ => None,
        };
        if let Some(start) = start
            && let Some(log) = Logarithm::starting_at(parent, start)
            && (log.base.is_some() || log.exponent.is_some())
            && (log.argument.id == child || log.has_script(child))
        {
            return Self {
                parent,
                start,
                argument: Some(log.argument),
            };
        }
        Self {
            parent,
            start: index,
            argument: None,
        }
    }

    pub(super) fn move_field(&self, child: u64, backward: bool) -> Position {
        // Scripts visit their editable subscript and superscript, skipping the
        // atom they decorate. Other nodes already store their fields in order.
        let owner = &self.parent.nodes[self.start];
        let mut fields = owner.children();
        if matches!(owner, Node::Script { .. }) {
            fields.next();
        }
        adjacent(fields.chain(self.argument), child, backward)
            .unwrap_or_else(|| self.outside(backward))
    }

    pub(super) fn outside(&self, backward: bool) -> Position {
        let width = 1 + usize::from(self.argument.is_some());
        Position {
            row: self.parent.id,
            offset: if backward {
                self.start
            } else {
                self.start + width
            },
        }
    }
}

/// In these structures horizontal arrows also follow the document field order.
/// Scripts have separate arrow behavior for entering and leaving their base.
pub(super) fn horizontal_fields(node: &Node) -> Option<impl DoubleEndedIterator<Item = &Row>> {
    match node {
        Node::Fraction { .. }
        | Node::Binomial { .. }
        | Node::Root { .. }
        | Node::Large { .. }
        | Node::Matrix { .. } => Some(node.children()),
        _ => None,
    }
}

pub(super) fn adjacent<'a>(
    mut fields: impl DoubleEndedIterator<Item = &'a Row>,
    current: u64,
    backward: bool,
) -> Option<Position> {
    let field = if backward {
        fields.rfind(|field| field.id == current)?;
        fields.next_back()?
    } else {
        fields.find(|field| field.id == current)?;
        fields.next()?
    };
    Some(Position {
        row: field.id,
        offset: if backward { field.nodes.len() } else { 0 },
    })
}
