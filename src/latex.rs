use crate::{
    Accent, Alphabet, Error, MatrixStyle, catalog,
    model::{LARGE_OPERATORS, LimitPlacement, MAX_BYTES, MAX_DEPTH, MAX_NODES, Node, Row, SYMBOLS},
};

pub(crate) fn export(row: &Row, strict: bool) -> Result<String, Error> {
    if strict && row.nodes.is_empty() {
        return Err(Error::new(
            "The expression contains an empty slot; save a draft to preserve incomplete input",
        ));
    }
    let mut out = String::new();
    for node in &row.nodes {
        match node {
            Node::Symbol { text } => {
                if text == "′" || text == "″" {
                    out.push_str(if text == "′" { "'" } else { "''" });
                    continue;
                }
                if let Some((command, _)) = SYMBOLS.iter().find(|(_, symbol)| symbol == text) {
                    out.push_str(&format!("\\{command} "));
                } else {
                    match text.as_str() {
                        "−" => out.push('-'),
                        " " => out.push_str("\\, "),
                        "\\" => out.push_str("\\backslash "),
                        "#" | "%" | "&" | "$" | "{" | "}" | "_" | "^" => {
                            out.push('\\');
                            out.push_str(text);
                        }
                        _ => out.push_str(text),
                    }
                }
            }
            Node::Function { name } => {
                if catalog::STANDARD_FUNCTIONS.contains(&name.as_str()) {
                    out.push_str(&format!("\\{name} "));
                } else {
                    out.push_str(&format!("\\operatorname{{{name}}}"));
                }
            }
            Node::Text { text } => {
                let mut escaped = String::new();
                for ch in text.chars() {
                    match ch {
                        '\\' => escaped.push_str("\\textbackslash "),
                        '^' => escaped.push_str("\\textasciicircum "),
                        '~' => escaped.push_str("\\textasciitilde "),
                        '{' | '}' | '#' | '%' | '&' | '$' | '_' => {
                            escaped.push('\\');
                            escaped.push(ch);
                        }
                        _ => escaped.push(ch),
                    }
                }
                out.push_str(&format!("\\text{{{escaped}}}"));
            }
            Node::Styled { alphabet, body } => out.push_str(&format!(
                "\\{}{{{}}}",
                alphabet.command(),
                export(body, strict)?
            )),
            Node::Fraction {
                numerator,
                denominator,
            } => out.push_str(&format!(
                "\\frac{{{}}}{{{}}}",
                export(numerator, strict)?,
                export(denominator, strict)?
            )),
            Node::Binomial { upper, lower } => out.push_str(&format!(
                "\\binom{{{}}}{{{}}}",
                export(upper, strict)?,
                export(lower, strict)?
            )),
            Node::Accent { accent, body } => {
                let name = match accent {
                    Accent::Hat => "widehat",
                    Accent::Bar => "overline",
                    Accent::Underline => "underline",
                    Accent::Vector => "vec",
                    Accent::Dot => "dot",
                    Accent::DoubleDot => "ddot",
                    Accent::Tilde => "widetilde",
                    Accent::Overbrace => "overbrace",
                    Accent::Underbrace => "underbrace",
                };
                out.push_str(&format!("\\{name}{{{}}}", export(body, strict)?));
            }
            Node::Stacked { base, over, under } => {
                let mut source = export(base, strict)?;
                if let Some(under) = under {
                    source = format!("\\underset{{{}}}{{{source}}}", export(under, strict)?);
                }
                if let Some(over) = over {
                    source = format!("\\overset{{{}}}{{{source}}}", export(over, strict)?);
                }
                out.push_str(&source);
            }
            Node::Root { index, body } => {
                out.push_str("\\sqrt");
                if let Some(index) = index {
                    out.push_str(&format!("[{}]", export(index, strict)?));
                }
                out.push_str(&format!("{{{}}}", export(body, strict)?));
            }
            Node::Script {
                placement,
                base,
                sub,
                sup,
            } => {
                let contents = export(base, strict)?;
                if base.nodes.len() != 1
                    || matches!(
                        base.nodes.first(),
                        Some(Node::Script { .. } | Node::Large { .. })
                    )
                {
                    out.push_str(&format!("{{{contents}}}"));
                } else {
                    out.push_str(&contents);
                }
                match placement {
                    LimitPlacement::Automatic => {}
                    LimitPlacement::AboveBelow => out.push_str("\\limits"),
                    LimitPlacement::Side => out.push_str("\\nolimits"),
                }
                if let Some(sub) = sub {
                    out.push_str(&format!("_{{{}}}", export(sub, strict)?));
                }
                if let Some(sup) = sup {
                    out.push_str(&format!("^{{{}}}", export(sup, strict)?));
                }
            }
            Node::Group {
                open,
                close,
                closed,
                ghost_open,
                body,
            } => {
                if strict && (!closed || *ghost_open) {
                    return Err(Error::new(
                        "A bracket still has an unconfirmed ghost partner",
                    ));
                }
                let delimiter = |c: char| match c {
                    '{' | '}' => format!("\\{c}"),
                    '⌊' => "\\lfloor ".into(),
                    '⌋' => "\\rfloor ".into(),
                    '⌈' => "\\lceil ".into(),
                    '⌉' => "\\rceil ".into(),
                    '⟨' => "\\langle ".into(),
                    '⟩' => "\\rangle ".into(),
                    '‖' => "\\lVert ".into(),
                    _ => c.to_string(),
                };
                out.push_str(&format!(
                    "\\left{}{}\\right{}",
                    if *ghost_open {
                        ".".into()
                    } else {
                        delimiter(*open)
                    },
                    export(body, strict)?,
                    if *closed {
                        if *close == '‖' {
                            "\\rVert ".into()
                        } else {
                            delimiter(*close)
                        }
                    } else {
                        ".".into()
                    }
                ));
            }
            Node::Large {
                placement,
                symbol,
                lower,
                upper,
            } => {
                let name = LARGE_OPERATORS
                    .iter()
                    .find(|(_, value)| value == symbol)
                    .expect("validated large operator")
                    .0;
                out.push_str(&format!("\\{name}"));
                match placement {
                    LimitPlacement::Automatic => {}
                    LimitPlacement::AboveBelow => out.push_str("\\limits"),
                    LimitPlacement::Side => out.push_str("\\nolimits"),
                }
                if let Some(lower) = lower {
                    out.push_str(&format!("_{{{}}}", export(lower, strict)?));
                }
                if let Some(upper) = upper {
                    out.push_str(&format!("^{{{}}}", export(upper, strict)?));
                }
                out.push(' ');
            }
            Node::Matrix { cells, style } => {
                let environment = style.environment();
                out.push_str(&format!("\\begin{{{environment}}}"));
                for (r, row) in cells.iter().enumerate() {
                    if r > 0 {
                        out.push_str("\\\\");
                    }
                    for (c, cell) in row.iter().enumerate() {
                        if c > 0 {
                            out.push('&');
                        }
                        out.push_str(&export(cell, strict)?);
                    }
                }
                out.push_str(&format!("\\end{{{environment}}}"));
            }
        }
    }
    Ok(out.trim_end().to_owned())
}

pub(crate) fn parse(source: &str, first_id: u64) -> Result<(Row, u64), Error> {
    if source.len() > MAX_BYTES {
        return Err(Error::new("LaTeX size limit exceeded"));
    }
    let mut input = source.trim();
    if input.starts_with("$$") && input.ends_with("$$") && input.len() >= 4 {
        input = &input[2..input.len() - 2];
    } else if input.starts_with('$') && input.ends_with('$') && input.len() >= 2 {
        input = &input[1..input.len() - 1];
    }
    let base = input.as_ptr() as usize - source.as_ptr() as usize;
    let mut parser = Parser {
        input,
        pos: 0,
        base,
        next_id: first_id,
        depth: 0,
        count: 0,
    };
    let row = parser.row(Stop::End)?;
    row.validate(parser.next_id)?;
    Ok((row, parser.next_id))
}
#[derive(Clone, Copy)]
enum Stop {
    End,
    Character(char),
    Group(char),
    Matrix,
    Stack,
}
struct Parser<'a> {
    input: &'a str,
    pos: usize,
    base: usize,
    next_id: u64,
    depth: usize,
    count: usize,
}
impl Parser<'_> {
    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }
    fn starts_command(&self, command: &str) -> bool {
        self.remaining()
            .strip_prefix(command)
            .is_some_and(|tail| !tail.starts_with(|c: char| c.is_ascii_alphabetic()))
    }
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }
    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }
    fn space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }
    fn error(&self, message: impl Into<String>) -> Error {
        Error::at(message, self.pos + self.base)
    }
    fn make_row(&mut self, nodes: Vec<Node>) -> Row {
        let id = self.next_id;
        self.next_id += 1;
        Row { id, nodes }
    }
    fn stops(&self, stop: Stop) -> bool {
        match stop {
            Stop::End => false,
            Stop::Character(c) => self.peek() == Some(c),
            Stop::Group(c) => {
                self.peek() == Some(c)
                    || self.starts_command("\\right")
                    || match c {
                        '⌋' => self.starts_command("\\rfloor"),
                        '⌉' => self.starts_command("\\rceil"),
                        '⟩' => self.starts_command("\\rangle"),
                        '‖' => self.starts_command("\\rVert"),
                        '|' => self.starts_command("\\rvert"),
                        _ => false,
                    }
            }
            Stop::Matrix => {
                self.peek() == Some('&')
                    || self.remaining().starts_with("\\\\")
                    || self.remaining().starts_with("\\end")
            }
            Stop::Stack => self.peek() == Some('}') || self.remaining().starts_with("\\\\"),
        }
    }
    fn row(&mut self, stop: Stop) -> Result<Row, Error> {
        self.depth += 1;
        if self.depth > MAX_DEPTH + 1 {
            return Err(self.error("LaTeX nesting limit exceeded"));
        }
        let mut row = self.make_row(vec![]);
        loop {
            self.space();
            if self.peek().is_none() || self.stops(stop) {
                break;
            }
            self.count += 1;
            if self.count > MAX_NODES {
                return Err(self.error("LaTeX node limit exceeded"));
            }
            match self.peek().expect("nonempty input") {
                '\\' if [
                    "\\displaystyle",
                    "\\textstyle",
                    "\\scriptstyle",
                    "\\limits",
                    "\\nolimits",
                ]
                .iter()
                .any(|command| {
                    self.remaining()
                        .strip_prefix(command)
                        .is_some_and(|tail| !tail.starts_with(|c: char| c.is_ascii_alphabetic()))
                }) =>
                {
                    let modifier_start = self.pos;
                    self.bump();
                    while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                        self.bump();
                    }
                    let modifier = &self.input[modifier_start..self.pos];
                    if matches!(modifier, "\\limits" | "\\nolimits") {
                        let placement = if modifier == "\\limits" {
                            LimitPlacement::AboveBelow
                        } else {
                            LimitPlacement::Side
                        };
                        match row.nodes.last_mut() {
                            Some(
                                Node::Script {
                                    placement: current, ..
                                }
                                | Node::Large {
                                    placement: current, ..
                                },
                            ) => *current = placement,
                            Some(Node::Function { .. }) => {
                                let operator = row.nodes.pop().expect("operator");
                                let base = self.make_row(vec![operator]);
                                row.nodes.push(Node::Script {
                                    placement,
                                    base,
                                    sub: None,
                                    sup: None,
                                });
                            }
                            _ => return Err(self.error("Limit placement requires an operator")),
                        }
                    }
                }
                '^' | '_' => {
                    let upper = self.bump() == Some('^');
                    let argument = self.argument()?;
                    let last = row
                        .nodes
                        .last_mut()
                        .ok_or_else(|| self.error("A script needs a preceding expression"))?;
                    match last {
                        Node::Script { sub, sup, .. } => {
                            let slot = if upper { sup } else { sub };
                            if slot.is_some() {
                                return Err(self.error("Repeated script on the same base"));
                            }
                            *slot = Some(argument);
                        }
                        Node::Large {
                            lower,
                            upper: upper_row,
                            ..
                        } => {
                            let slot = if upper { upper_row } else { lower };
                            if slot.is_some() {
                                return Err(self.error("Repeated bound on the same operator"));
                            }
                            *slot = Some(argument);
                        }
                        _ => {
                            let base_node = row.nodes.pop().expect("script base");
                            let base = self.make_row(vec![base_node]);
                            row.nodes.push(if upper {
                                Node::Script {
                                    placement: Default::default(),
                                    base,
                                    sup: Some(argument),
                                    sub: None,
                                }
                            } else {
                                Node::Script {
                                    placement: Default::default(),
                                    base,
                                    sub: Some(argument),
                                    sup: None,
                                }
                            });
                        }
                    }
                }
                '{' => {
                    self.bump();
                    let grouped = self.row(Stop::Character('}'))?;
                    self.expect('}')?;
                    self.space();
                    if matches!(self.peek(), Some('^' | '_')) {
                        row.nodes.push(Node::Script {
                            placement: Default::default(),
                            base: grouped,
                            sub: None,
                            sup: None,
                        });
                    } else {
                        row.nodes.extend(grouped.nodes);
                    }
                }
                '}' | ')' | ']' => return Err(self.error("Unexpected closing delimiter")),
                _ => row.nodes.push(self.atom()?),
            }
        }
        self.depth -= 1;
        Ok(row)
    }
    fn expect(&mut self, expected: char) -> Result<(), Error> {
        if self.bump() != Some(expected) {
            return Err(self.error(format!("Expected '{expected}'")));
        }
        Ok(())
    }
    fn argument(&mut self) -> Result<Row, Error> {
        self.space();
        if self.peek() == Some('{') {
            self.bump();
            let row = self.row(Stop::Character('}'))?;
            self.expect('}')?;
            Ok(row)
        } else if self.peek().is_none() {
            Err(self.error("Missing LaTeX argument"))
        } else {
            let node = self.atom()?;
            Ok(self.make_row(vec![node]))
        }
    }
    fn raw_argument(&mut self) -> Result<String, Error> {
        self.space();
        self.expect('{')?;
        let mut text = String::new();
        let mut depth = 1;
        while let Some(ch) = self.bump() {
            match ch {
                '{' => {
                    depth += 1;
                    text.push(ch);
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(text);
                    }
                    text.push(ch);
                }
                '\\' => {
                    if let Some((command, literal)) = [
                        ("backslash ", '\\'),
                        ("textbackslash ", '\\'),
                        ("textasciicircum ", '^'),
                        ("textasciitilde ", '~'),
                    ]
                    .into_iter()
                    .find(|(command, _)| self.remaining().starts_with(command))
                    {
                        self.pos += command.len();
                        text.push(literal);
                    } else {
                        text.push(
                            self.bump()
                                .ok_or_else(|| self.error("Incomplete text escape"))?,
                        );
                    }
                }
                _ => text.push(ch),
            }
        }
        Err(self.error("Unclosed text argument"))
    }
    fn delimiter(&mut self) -> Result<char, Error> {
        self.space();
        let ch = self.bump().ok_or_else(|| self.error("Missing delimiter"))?;
        if ch == '\\' {
            let start = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                self.bump();
            }
            let name = &self.input[start..self.pos];
            if name.is_empty() {
                return self
                    .bump()
                    .ok_or_else(|| self.error("Missing escaped delimiter"));
            }
            match name {
                "lfloor" => Ok('⌊'),
                "rfloor" => Ok('⌋'),
                "lceil" => Ok('⌈'),
                "rceil" => Ok('⌉'),
                "langle" => Ok('⟨'),
                "rangle" => Ok('⟩'),
                "lVert" | "rVert" | "Vert" => Ok('‖'),
                "lvert" | "rvert" | "vert" => Ok('|'),
                "lbrace" => Ok('{'),
                "rbrace" => Ok('}'),
                _ => Err(self.error("Unsupported named delimiter")),
            }
        } else {
            Ok(ch)
        }
    }
    fn group(&mut self, open: char, explicit: bool) -> Result<Node, Error> {
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '|' => '|',
            '⌊' => '⌋',
            '⌈' => '⌉',
            '⟨' => '⟩',
            '‖' => '‖',
            '.' => '\0',
            _ => return Err(self.error("Unsupported opening delimiter")),
        };
        let body = self.row(Stop::Group(close))?;
        let mut closed = false;
        let mut actual_close = close;
        if self.starts_command("\\right") {
            self.pos += "\\right".len();
            let delimiter = self.delimiter()?;
            if delimiter != '.' {
                closed = true;
                actual_close = delimiter;
            }
        } else if self.peek() == Some(close) {
            self.bump();
            closed = true;
        } else if explicit {
            return Err(self.error("Missing \\right delimiter"));
        }
        let ghost_open = open == '.';
        let open = if ghost_open {
            match actual_close {
                ')' => '(',
                ']' => '[',
                '}' => '{',
                '|' => '|',
                '⌋' => '⌊',
                '⌉' => '⌈',
                '⟩' => '⟨',
                '‖' => '‖',
                _ => {
                    return Err(
                        self.error("A ghost opening bracket needs a supported closing delimiter")
                    );
                }
            }
        } else {
            open
        };
        Ok(Node::Group {
            open,
            close: actual_close,
            closed,
            ghost_open,
            body,
        })
    }
    fn atom(&mut self) -> Result<Node, Error> {
        let ch = self.bump().ok_or_else(|| self.error("Expected an atom"))?;
        match ch {
            '(' | '[' | '|' | '⌊' | '⌈' | '⟨' | '‖' => self.group(ch, false),
            '\\' => self.command(),
            '{' | '}' | '^' | '_' | '$' => Err(self.error("Unexpected LaTeX syntax")),
            '-' => Ok(Node::symbol("−")),
            '*' => Ok(Node::symbol("·")),
            '\'' => Ok(Node::symbol("′")),
            _ if ch.is_control() => Err(self.error("Control character in mathematical input")),
            _ => Ok(Node::symbol(ch.to_string())),
        }
    }
    fn command(&mut self) -> Result<Node, Error> {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.bump();
        }
        let command = self.input[start..self.pos].to_owned();
        if command.is_empty() {
            let ch = self
                .bump()
                .ok_or_else(|| self.error("Incomplete LaTeX command"))?;
            return match ch {
                '%' | '#' | '&' | '$' | '{' | '}' | '_' | '^' | ' ' => {
                    Ok(Node::symbol(ch.to_string()))
                }
                ',' | ';' | ':' | '!' => Ok(Node::symbol(" ")),
                _ => Err(self.error("Unsupported escaped character")),
            };
        }
        match command.as_str() {
            "binom" | "dbinom" | "tbinom" => {
                let upper = self.argument()?;
                let lower = self.argument()?;
                Ok(Node::Binomial { upper, lower })
            }
            "hat" | "widehat" | "bar" | "overline" | "underline" | "vec" | "overrightarrow"
            | "dot" | "ddot" | "tilde" | "widetilde" | "overbrace" | "underbrace" => {
                let accent = match command.as_str() {
                    "hat" | "widehat" => Accent::Hat,
                    "bar" | "overline" => Accent::Bar,
                    "underline" => Accent::Underline,
                    "vec" | "overrightarrow" => Accent::Vector,
                    "dot" => Accent::Dot,
                    "ddot" => Accent::DoubleDot,
                    "tilde" | "widetilde" => Accent::Tilde,
                    "overbrace" => Accent::Overbrace,
                    _ => Accent::Underbrace,
                };
                let body = self.argument()?;
                Ok(Node::Accent { accent, body })
            }
            "overset" | "underset" | "stackrel" => {
                let annotation = self.argument()?;
                let base = self.argument()?;
                let over = command != "underset";
                Ok(Node::Stacked {
                    base,
                    over: over.then(|| annotation.clone()),
                    under: (!over).then_some(annotation),
                })
            }
            "lfloor" | "lceil" | "langle" | "lVert" | "lvert" => {
                let open = match command.as_str() {
                    "lfloor" => '⌊',
                    "lceil" => '⌈',
                    "langle" => '⟨',
                    "lVert" => '‖',
                    _ => '|',
                };
                let close_command = match command.as_str() {
                    "lfloor" => "\\rfloor",
                    "lceil" => "\\rceil",
                    "langle" => "\\rangle",
                    "lVert" => "\\rVert",
                    _ => "\\rvert",
                };
                // Normalize named delimiters through the same scalable group parser.
                let body = self.row(Stop::Group(match open {
                    '⌊' => '⌋',
                    '⌈' => '⌉',
                    '⟨' => '⟩',
                    '‖' => '‖',
                    _ => '|',
                }))?;
                if !self.remaining().starts_with(close_command) {
                    return Err(self.error("Missing closing delimiter"));
                }
                self.pos += close_command.len();
                Ok(Node::Group {
                    open,
                    close: match open {
                        '⌊' => '⌋',
                        '⌈' => '⌉',
                        '⟨' => '⟩',
                        '‖' => '‖',
                        _ => '|',
                    },
                    closed: true,
                    ghost_open: false,
                    body,
                })
            }
            "mathbb" | "mathcal" | "mathfrak" | "mathbf" | "mathit" | "mathsf" | "mathtt" => {
                let alphabet = match command.as_str() {
                    "mathbb" => Alphabet::DoubleStruck,
                    "mathcal" => Alphabet::Calligraphic,
                    "mathfrak" => Alphabet::Fraktur,
                    "mathbf" => Alphabet::Bold,
                    "mathit" => Alphabet::Italic,
                    "mathsf" => Alphabet::SansSerif,
                    _ => Alphabet::Monospace,
                };
                let body = self.argument()?;
                Ok(Node::Styled { alphabet, body })
            }
            "substack" => {
                self.space();
                self.expect('{')?;
                let mut cells = vec![];
                loop {
                    cells.push(vec![self.row(Stop::Stack)?]);
                    if self.peek() == Some('}') {
                        self.bump();
                        break;
                    }
                    if self.remaining().starts_with("\\\\") {
                        self.pos += 2;
                    } else {
                        return Err(self.error("Unclosed substack"));
                    }
                    if cells.len() > 16 {
                        return Err(self.error("Substack row limit exceeded"));
                    }
                }
                Ok(Node::Matrix {
                    cells,
                    style: MatrixStyle::Plain,
                })
            }
            "frac" | "dfrac" | "tfrac" => {
                let numerator = self.argument()?;
                let denominator = self.argument()?;
                Ok(Node::Fraction {
                    numerator,
                    denominator,
                })
            }
            "sqrt" => {
                self.space();
                let index = if self.peek() == Some('[') {
                    self.bump();
                    let row = self.row(Stop::Character(']'))?;
                    self.expect(']')?;
                    Some(row)
                } else {
                    None
                };
                let body = self.argument()?;
                Ok(Node::Root { index, body })
            }
            "left" => {
                let open = self.delimiter()?;
                self.group(open, true)
            }
            "text" | "mathrm" => Ok(Node::Text {
                text: self.raw_argument()?,
            }),
            "operatorname" => {
                let limits = self.peek() == Some('*');
                if limits {
                    self.bump();
                }
                let name = self.raw_argument()?;
                if !name.chars().all(|c| c.is_ascii_alphabetic()) || name.is_empty() {
                    return Err(self.error("Operator names must contain letters"));
                }
                if limits {
                    let base = self.make_row(vec![Node::Function { name }]);
                    Ok(Node::Script {
                        placement: LimitPlacement::AboveBelow,
                        base,
                        sub: None,
                        sup: None,
                    })
                } else {
                    Ok(Node::Function { name })
                }
            }
            _ if LARGE_OPERATORS.iter().any(|(name, _)| *name == command) => Ok(Node::Large {
                placement: Default::default(),
                symbol: LARGE_OPERATORS
                    .iter()
                    .find(|(name, _)| *name == command)
                    .expect("matched operator")
                    .1,
                lower: None,
                upper: None,
            }),
            "begin" => {
                let name = self.raw_argument()?;
                let style = match name.as_str() {
                    "bmatrix" => MatrixStyle::Brackets,
                    "pmatrix" => MatrixStyle::Parentheses,
                    "Bmatrix" => MatrixStyle::Braces,
                    "vmatrix" => MatrixStyle::Bars,
                    "Vmatrix" => MatrixStyle::DoubleBars,
                    "matrix" | "gathered" => MatrixStyle::Plain,
                    "cases" => MatrixStyle::Cases,
                    "aligned" => MatrixStyle::Aligned,
                    _ => return Err(self.error("Unsupported matrix environment")),
                };
                let mut cells = vec![vec![]];
                loop {
                    let cell = self.row(Stop::Matrix)?;
                    cells.last_mut().expect("matrix row").push(cell);
                    self.space();
                    if self.peek() == Some('&') {
                        self.bump();
                    } else if self.remaining().starts_with("\\\\") {
                        self.pos += 2;
                        cells.push(vec![]);
                    } else if self.remaining().starts_with("\\end") {
                        self.pos += 4;
                        if self.raw_argument()? != name {
                            return Err(self.error("Mismatched matrix environment"));
                        }
                        break;
                    } else {
                        return Err(self.error("Unclosed matrix environment"));
                    }
                    if cells.len() > 16 || cells.last().expect("matrix row").len() > 16 {
                        return Err(self.error("Matrix dimension limit exceeded"));
                    }
                }
                if style == MatrixStyle::Aligned {
                    let columns = cells.iter().map(Vec::len).max().unwrap_or(1);
                    for row in &mut cells {
                        while row.len() < columns {
                            row.push(self.make_row(vec![]));
                        }
                    }
                }
                Ok(Node::Matrix { cells, style })
            }
            "backslash" => Ok(Node::symbol("\\")),
            _ => {
                if let Some((_, value)) = SYMBOLS.iter().find(|(name, _)| *name == command) {
                    return Ok(Node::symbol(*value));
                }
                if catalog::STANDARD_FUNCTIONS.contains(&command.as_str())
                    || catalog::function_names().any(|name| name == command)
                {
                    return Ok(Node::Function { name: command });
                }
                Err(Error::at(
                    format!("Unsupported LaTeX command \\{command}; input was preserved"),
                    start - 1 + self.base,
                ))
            }
        }
    }
}
