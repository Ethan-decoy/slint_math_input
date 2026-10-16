use crate::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub(crate) const MAX_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_DRAFT_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_NODES: usize = 10_000;
pub(crate) const MAX_DEPTH: usize = 64;

/// Mathematical alphabet applied to an editable expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alphabet {
    /// Bold mathematical letters and digits.
    Bold,
    /// Blackboard bold letters and digits.
    DoubleStruck,
    /// Calligraphic letters.
    Calligraphic,
    /// Fraktur letters.
    Fraktur,
    /// Italic mathematical letters.
    Italic,
    /// Sans serif mathematical letters and digits.
    SansSerif,
    /// Monospaced mathematical letters and digits.
    Monospace,
}
impl Alphabet {
    pub(crate) fn command(self) -> &'static str {
        match self {
            Self::Bold => "mathbf",
            Self::DoubleStruck => "mathbb",
            Self::Calligraphic => "mathcal",
            Self::Fraktur => "mathfrak",
            Self::Italic => "mathit",
            Self::SansSerif => "mathsf",
            Self::Monospace => "mathtt",
        }
    }
    pub(crate) fn apply(self, row: &mut Row) {
        for node in &mut row.nodes {
            if let Node::Symbol { text } = node {
                let key = format!("{}{{{text}}}", self.command());
                if let Some((_, symbol)) = SYMBOLS.iter().find(|(name, _)| *name == key) {
                    *text = (*symbol).into();
                }
            } else if !matches!(node, Node::Styled { .. }) {
                for child in node.children_mut() {
                    self.apply(child);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum LimitPlacement {
    #[default]
    Automatic,
    AboveBelow,
    Side,
}
impl LimitPlacement {
    fn is_auto(&self) -> bool {
        *self == Self::Automatic
    }
}

/// Scalable mathematical decoration applied to an editable expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accent {
    /// A hat above the expression.
    Hat,
    /// A bar above the expression.
    Bar,
    /// A line below the expression.
    Underline,
    /// A vector arrow above the expression.
    Vector,
    /// A dot above the expression.
    Dot,
    /// Two dots above the expression.
    DoubleDot,
    /// A tilde above the expression.
    Tilde,
    /// A brace above the expression.
    Overbrace,
    /// A brace below the expression.
    Underbrace,
}

/// Delimiters and alignment used for a rectangular array of editable cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatrixStyle {
    /// Square brackets.
    #[default]
    Brackets,
    /// Parentheses.
    Parentheses,
    /// Braces on both sides.
    Braces,
    /// Determinant bars.
    Bars,
    /// Double vertical bars.
    DoubleBars,
    /// No delimiters.
    Plain,
    /// A left brace and two left-aligned columns.
    Cases,
    /// Alternating right/left aligned equation columns, without delimiters.
    Aligned,
}
impl MatrixStyle {
    pub(crate) fn environment(self) -> &'static str {
        match self {
            Self::Brackets => "bmatrix",
            Self::Parentheses => "pmatrix",
            Self::Braces => "Bmatrix",
            Self::Bars => "vmatrix",
            Self::DoubleBars => "Vmatrix",
            Self::Plain => "matrix",
            Self::Cases => "cases",
            Self::Aligned => "aligned",
        }
    }
    pub(crate) fn delimiters(self) -> (Option<char>, Option<char>) {
        match self {
            Self::Brackets => (Some('['), Some(']')),
            Self::Parentheses => (Some('('), Some(')')),
            Self::Braces => (Some('{'), Some('}')),
            Self::Bars => (Some('|'), Some('|')),
            Self::DoubleBars => (Some('‖'), Some('‖')),
            Self::Cases => (Some('{'), None),
            Self::Plain | Self::Aligned => (None, None),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Row {
    pub id: u64,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Node {
    Symbol {
        text: String,
    },
    Function {
        name: String,
    },
    Text {
        text: String,
    },
    Fraction {
        numerator: Row,
        denominator: Row,
    },
    Binomial {
        upper: Row,
        lower: Row,
    },
    Accent {
        accent: Accent,
        body: Row,
    },
    Styled {
        alphabet: Alphabet,
        body: Row,
    },
    Stacked {
        base: Row,
        over: Option<Row>,
        under: Option<Row>,
    },
    Root {
        index: Option<Row>,
        body: Row,
    },
    Script {
        #[serde(default, skip_serializing_if = "LimitPlacement::is_auto")]
        placement: LimitPlacement,
        base: Row,
        sub: Option<Row>,
        sup: Option<Row>,
    },
    Group {
        open: char,
        close: char,
        closed: bool,
        #[serde(default)]
        ghost_open: bool,
        body: Row,
    },
    Large {
        #[serde(default, skip_serializing_if = "LimitPlacement::is_auto")]
        placement: LimitPlacement,
        symbol: char,
        lower: Option<Row>,
        upper: Option<Row>,
    },
    Matrix {
        cells: Vec<Vec<Row>>,
        #[serde(default)]
        style: MatrixStyle,
    },
}

impl Node {
    pub fn symbol(text: impl Into<String>) -> Self {
        Self::Symbol { text: text.into() }
    }
    pub fn children(&self) -> Vec<&Row> {
        match self {
            Self::Symbol { .. } | Self::Function { .. } | Self::Text { .. } => vec![],
            Self::Fraction {
                numerator,
                denominator,
            } => vec![numerator, denominator],
            Self::Binomial { upper, lower } => vec![upper, lower],
            Self::Accent { body, .. } | Self::Styled { body, .. } => vec![body],
            Self::Stacked { base, over, under } => {
                std::iter::once(base).chain(over).chain(under).collect()
            }
            Self::Root { index, body } => index.iter().chain(std::iter::once(body)).collect(),
            Self::Script {
                placement: _,
                base,
                sub,
                sup,
            } => std::iter::once(base).chain(sub).chain(sup).collect(),
            Self::Group { body, .. } => vec![body],
            Self::Large { lower, upper, .. } => lower.iter().chain(upper).collect(),
            Self::Matrix { cells, .. } => cells.iter().flatten().collect(),
        }
    }
    pub fn children_mut(&mut self) -> Vec<&mut Row> {
        match self {
            Self::Symbol { .. } | Self::Function { .. } | Self::Text { .. } => vec![],
            Self::Fraction {
                numerator,
                denominator,
            } => vec![numerator, denominator],
            Self::Binomial { upper, lower } => vec![upper, lower],
            Self::Accent { body, .. } | Self::Styled { body, .. } => vec![body],
            Self::Stacked { base, over, under } => {
                std::iter::once(base).chain(over).chain(under).collect()
            }
            Self::Root { index, body } => index.iter_mut().chain(std::iter::once(body)).collect(),
            Self::Script {
                placement: _,
                base,
                sub,
                sup,
            } => std::iter::once(base).chain(sub).chain(sup).collect(),
            Self::Group { body, .. } => vec![body],
            Self::Large { lower, upper, .. } => lower.iter_mut().chain(upper).collect(),
            Self::Matrix { cells, .. } => cells.iter_mut().flatten().collect(),
        }
    }
    pub fn entry(&self) -> Option<&Row> {
        match self {
            Self::Fraction { numerator, .. } => Some(numerator),
            Self::Script {
                placement: _,
                base,
                sup,
                sub,
            } => sup.as_ref().or(sub.as_ref()).or(Some(base)),
            Self::Large { lower, upper, .. } => lower.as_ref().or(upper.as_ref()),
            Self::Stacked { over, under, base } => over.as_ref().or(under.as_ref()).or(Some(base)),
            Self::Root { body, .. } | Self::Group { body, .. } => Some(body),
            _ => self.children().first().copied(),
        }
    }
    pub fn is_boundary(&self) -> bool {
        matches!(self, Self::Symbol { text } if text.chars().any(|c| "+−-=<>≤≥≠×·÷,:; ±∓≈≃≅≡∝≪≫∼∈∉∋⊂⊃⊆⊇⊊⊋∪∩∖∧∨⊕⊗⊙⊥∥→←↔⇒⇐⇔↦".contains(c)))
            || matches!(self, Self::Large { .. })
            || matches!(self, Self::Script { base, .. } if base.centers_limits())
    }
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Symbol { text } => Some(text),
            _ => None,
        }
    }
    pub fn into_contents(self) -> Vec<Node> {
        match self {
            Self::Fraction {
                numerator,
                denominator,
            } => numerator
                .nodes
                .into_iter()
                .chain(denominator.nodes)
                .collect(),
            Self::Binomial { upper, lower } => upper.nodes.into_iter().chain(lower.nodes).collect(),
            Self::Accent { body, .. } | Self::Styled { body, .. } => body.nodes,
            Self::Stacked { base, over, under } => base
                .nodes
                .into_iter()
                .chain(over.into_iter().flat_map(|r| r.nodes))
                .chain(under.into_iter().flat_map(|r| r.nodes))
                .collect(),
            Self::Root { index, body } => index
                .into_iter()
                .flat_map(|r| r.nodes)
                .chain(body.nodes)
                .collect(),
            Self::Script {
                placement: _,
                base,
                sub,
                sup,
            } => base
                .nodes
                .into_iter()
                .chain(sub.into_iter().flat_map(|r| r.nodes))
                .chain(sup.into_iter().flat_map(|r| r.nodes))
                .collect(),
            Self::Group { body, .. } => body.nodes,
            Self::Large { lower, upper, .. } => lower
                .into_iter()
                .chain(upper)
                .flat_map(|r| r.nodes)
                .collect(),
            Self::Matrix { cells, .. } => {
                cells.into_iter().flatten().flat_map(|r| r.nodes).collect()
            }
            leaf => vec![leaf],
        }
    }
}
impl Row {
    pub(crate) fn centers_limits(&self) -> bool {
        matches!(self.nodes.as_slice(), [Node::Function {name}] if matches!(name.as_str(), "lim"|"limsup"|"liminf"|"sup"|"inf"|"max"|"min"))
            || matches!(
                self.nodes.as_slice(),
                [Node::Accent {
                    accent: Accent::Overbrace | Accent::Underbrace,
                    ..
                }]
            )
    }
    pub fn find(&self, id: u64) -> Option<&Self> {
        if self.id == id {
            return Some(self);
        }
        self.nodes
            .iter()
            .flat_map(Node::children)
            .find_map(|r| r.find(id))
    }
    pub fn find_mut(&mut self, id: u64) -> Option<&mut Self> {
        if self.id == id {
            return Some(self);
        }
        self.nodes
            .iter_mut()
            .flat_map(Node::children_mut)
            .find_map(|r| r.find_mut(id))
    }
    pub fn parent(&self, id: u64) -> Option<(u64, usize)> {
        for (i, node) in self.nodes.iter().enumerate() {
            for child in node.children() {
                if child.id == id {
                    return Some((self.id, i));
                }
                if let Some(parent) = child.parent(id) {
                    return Some(parent);
                }
            }
        }
        None
    }
    pub fn ancestry(&self, id: u64) -> Vec<u64> {
        let mut result = vec![id];
        while let Some((parent, _)) = self.parent(*result.last().expect("nonempty ancestry")) {
            result.push(parent);
        }
        result
    }
    pub fn validate(&self, next_id: u64) -> Result<(), Error> {
        fn visit(
            row: &Row,
            depth: usize,
            ids: &mut HashSet<u64>,
            count: &mut usize,
            bytes: &mut usize,
        ) -> Result<(), Error> {
            if depth > MAX_DEPTH {
                return Err(Error::new("Expression nesting limit exceeded"));
            }
            if !ids.insert(row.id) {
                return Err(Error::new("Duplicate draft row identifier"));
            }
            for node in &row.nodes {
                *count += 1;
                if *count > MAX_NODES {
                    return Err(Error::new("Expression node limit exceeded"));
                }
                match node {
                    Node::Symbol { text } => {
                        if text.chars().count() != 1 || text.chars().any(char::is_control) {
                            return Err(Error::new("Invalid symbol in draft"));
                        }
                        *bytes += text.len();
                    }
                    Node::Function { name } => {
                        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphabetic()) {
                            return Err(Error::new("Invalid function name"));
                        }
                        *bytes += name.len();
                    }
                    Node::Text { text } => {
                        if text.chars().any(char::is_control) {
                            return Err(Error::new("Control characters are not allowed in text"));
                        }
                        *bytes += text.len();
                    }
                    Node::Group { open, close, .. }
                        if !matches!(
                            (*open, *close),
                            ('(', ')')
                                | ('[', ']')
                                | ('{', '}')
                                | ('|', '|')
                                | ('(', ']')
                                | ('[', ')')
                                | ('⌊', '⌋')
                                | ('⌈', '⌉')
                                | ('⟨', '⟩')
                                | ('‖', '‖')
                        ) =>
                    {
                        return Err(Error::new("Invalid delimiters in draft"));
                    }
                    Node::Large { symbol, .. } if !"∑∏∫∮∬∭⋃⋂∐⨁⨂".contains(*symbol) =>
                    {
                        return Err(Error::new("Unsupported large operator"));
                    }
                    Node::Matrix { cells, .. }
                        if cells.is_empty()
                            || cells.len() > 16
                            || cells[0].is_empty()
                            || cells[0].len() > 16
                            || cells.iter().any(|r| r.len() != cells[0].len()) =>
                    {
                        return Err(Error::new("Invalid matrix dimensions"));
                    }
                    Node::Matrix {
                        cells,
                        style: MatrixStyle::Cases,
                    } if cells[0].len() != 2 => {
                        return Err(Error::new("Cases need two columns"));
                    }
                    Node::Stacked {
                        over: None,
                        under: None,
                        ..
                    } => return Err(Error::new("An annotation needs an above or below slot")),
                    _ => {}
                }
                if *bytes > MAX_BYTES {
                    return Err(Error::new("Expression size limit exceeded"));
                }
                for child in node.children() {
                    visit(child, depth + 1, ids, count, bytes)?;
                }
            }
            Ok(())
        }
        let mut ids = HashSet::new();
        visit(self, 0, &mut ids, &mut 0, &mut 0)?;
        if next_id > 1_000_000_000 || ids.iter().any(|id| *id >= next_id) {
            return Err(Error::new("Invalid next row identifier"));
        }
        Ok(())
    }
}
pub(crate) const SYMBOLS: &[(&str, &str)] = &[
    ("alpha", "α"),
    ("beta", "β"),
    ("gamma", "γ"),
    ("delta", "δ"),
    ("epsilon", "ϵ"),
    ("theta", "θ"),
    ("lambda", "λ"),
    ("mu", "μ"),
    ("nu", "ν"),
    ("xi", "ξ"),
    ("pi", "π"),
    ("rho", "ρ"),
    ("sigma", "σ"),
    ("tau", "τ"),
    ("phi", "ϕ"),
    ("chi", "χ"),
    ("psi", "ψ"),
    ("omega", "ω"),
    ("Gamma", "Γ"),
    ("Delta", "Δ"),
    ("Theta", "Θ"),
    ("Lambda", "Λ"),
    ("Pi", "Π"),
    ("Sigma", "Σ"),
    ("Omega", "Ω"),
    ("infty", "∞"),
    ("cdot", "·"),
    ("times", "×"),
    ("div", "÷"),
    ("pm", "±"),
    ("mp", "∓"),
    ("le", "≤"),
    ("leq", "≤"),
    ("ge", "≥"),
    ("geq", "≥"),
    ("ne", "≠"),
    ("neq", "≠"),
    ("approx", "≈"),
    ("to", "→"),
    ("partial", "∂"),
    ("nabla", "∇"),
    ("in", "∈"),
    ("notin", "∉"),
    ("cup", "∪"),
    ("cap", "∩"),
    ("emptyset", "∅"),
    ("forall", "∀"),
    ("exists", "∃"),
    ("ldots", "…"),
    ("degree", "°"),
    ("zeta", "ζ"),
    ("eta", "η"),
    ("iota", "ι"),
    ("kappa", "κ"),
    ("omicron", "ο"),
    ("upsilon", "υ"),
    ("varepsilon", "ε"),
    ("vartheta", "ϑ"),
    ("varkappa", "ϰ"),
    ("varpi", "ϖ"),
    ("varrho", "ϱ"),
    ("varsigma", "ς"),
    ("varphi", "φ"),
    ("Xi", "Ξ"),
    ("Upsilon", "Υ"),
    ("Phi", "Φ"),
    ("Psi", "Ψ"),
    ("simeq", "≃"),
    ("cong", "≅"),
    ("equiv", "≡"),
    ("propto", "∝"),
    ("ll", "≪"),
    ("gg", "≫"),
    ("sim", "∼"),
    ("ni", "∋"),
    ("subset", "⊂"),
    ("supset", "⊃"),
    ("subseteq", "⊆"),
    ("supseteq", "⊇"),
    ("subsetneq", "⊊"),
    ("supsetneq", "⊋"),
    ("setminus", "∖"),
    ("nexists", "∄"),
    ("land", "∧"),
    ("lor", "∨"),
    ("neg", "¬"),
    ("oplus", "⊕"),
    ("otimes", "⊗"),
    ("odot", "⊙"),
    ("perp", "⊥"),
    ("parallel", "∥"),
    ("angle", "∠"),
    ("leftarrow", "←"),
    ("rightarrow", "→"),
    ("longrightarrow", "⟶"),
    ("longleftarrow", "⟵"),
    ("longleftrightarrow", "⟷"),
    ("quad", " "),
    ("enspace", " "),
    ("mathbf{0}", "𝟎"),
    ("mathbf{1}", "𝟏"),
    ("mathbf{2}", "𝟐"),
    ("mathbf{3}", "𝟑"),
    ("mathbf{4}", "𝟒"),
    ("mathbf{5}", "𝟓"),
    ("mathbf{6}", "𝟔"),
    ("mathbf{7}", "𝟕"),
    ("mathbf{8}", "𝟖"),
    ("mathbf{9}", "𝟗"),
    ("mathbb{0}", "𝟘"),
    ("mathbb{1}", "𝟙"),
    ("mathbb{2}", "𝟚"),
    ("mathbb{3}", "𝟛"),
    ("mathbb{4}", "𝟜"),
    ("mathbb{5}", "𝟝"),
    ("mathbb{6}", "𝟞"),
    ("mathbb{7}", "𝟟"),
    ("mathbb{8}", "𝟠"),
    ("mathbb{9}", "𝟡"),
    ("mathsf{0}", "𝟢"),
    ("mathsf{1}", "𝟣"),
    ("mathsf{2}", "𝟤"),
    ("mathsf{3}", "𝟥"),
    ("mathsf{4}", "𝟦"),
    ("mathsf{5}", "𝟧"),
    ("mathsf{6}", "𝟨"),
    ("mathsf{7}", "𝟩"),
    ("mathsf{8}", "𝟪"),
    ("mathsf{9}", "𝟫"),
    ("mathtt{0}", "𝟶"),
    ("mathtt{1}", "𝟷"),
    ("mathtt{2}", "𝟸"),
    ("mathtt{3}", "𝟹"),
    ("mathtt{4}", "𝟺"),
    ("mathtt{5}", "𝟻"),
    ("mathtt{6}", "𝟼"),
    ("mathtt{7}", "𝟽"),
    ("mathtt{8}", "𝟾"),
    ("mathtt{9}", "𝟿"),
    ("leftrightarrow", "↔"),
    ("Rightarrow", "⇒"),
    ("Leftarrow", "⇐"),
    ("Leftrightarrow", "⇔"),
    ("mapsto", "↦"),
    ("uparrow", "↑"),
    ("downarrow", "↓"),
    ("nearrow", "↗"),
    ("searrow", "↘"),
    ("circ", "∘"),
    ("ast", "∗"),
    ("star", "⋆"),
    ("bullet", "•"),
    ("uplus", "⊎"),
    ("therefore", "∴"),
    ("because", "∵"),
    ("coloneqq", "≔"),
    ("cdots", "⋯"),
    ("vdots", "⋮"),
    ("ddots", "⋱"),
    ("aleph", "ℵ"),
    ("hbar", "ℏ"),
    ("ell", "ℓ"),
    ("Re", "ℜ"),
    ("Im", "ℑ"),
    ("prime", "′"),
    ("dprime", "″"),
    ("mathbb{N}", "ℕ"),
    ("mathbb{Z}", "ℤ"),
    ("mathbb{Q}", "ℚ"),
    ("mathbb{R}", "ℝ"),
    ("mathbb{C}", "ℂ"),
    ("mathbb{P}", "ℙ"),
    ("mathbb{H}", "ℍ"),
    ("mathbb{A}", "𝔸"),
    ("mathbb{B}", "𝔹"),
    ("mathbb{D}", "𝔻"),
    ("mathbb{E}", "𝔼"),
    ("mathbb{F}", "𝔽"),
    ("mathbb{G}", "𝔾"),
    ("mathbb{I}", "𝕀"),
    ("mathbb{J}", "𝕁"),
    ("mathbb{K}", "𝕂"),
    ("mathbb{L}", "𝕃"),
    ("mathbb{M}", "𝕄"),
    ("mathbb{O}", "𝕆"),
    ("mathbb{S}", "𝕊"),
    ("mathbb{T}", "𝕋"),
    ("mathbb{U}", "𝕌"),
    ("mathbb{V}", "𝕍"),
    ("mathbb{W}", "𝕎"),
    ("mathbb{X}", "𝕏"),
    ("mathbb{Y}", "𝕐"),
    ("mathbb{a}", "𝕒"),
    ("mathbb{b}", "𝕓"),
    ("mathbb{c}", "𝕔"),
    ("mathbb{d}", "𝕕"),
    ("mathbb{e}", "𝕖"),
    ("mathbb{f}", "𝕗"),
    ("mathbb{g}", "𝕘"),
    ("mathbb{h}", "𝕙"),
    ("mathbb{i}", "𝕚"),
    ("mathbb{j}", "𝕛"),
    ("mathbb{k}", "𝕜"),
    ("mathbb{l}", "𝕝"),
    ("mathbb{m}", "𝕞"),
    ("mathbb{n}", "𝕟"),
    ("mathbb{o}", "𝕠"),
    ("mathbb{p}", "𝕡"),
    ("mathbb{q}", "𝕢"),
    ("mathbb{r}", "𝕣"),
    ("mathbb{s}", "𝕤"),
    ("mathbb{t}", "𝕥"),
    ("mathbb{u}", "𝕦"),
    ("mathbb{v}", "𝕧"),
    ("mathbb{w}", "𝕨"),
    ("mathbb{x}", "𝕩"),
    ("mathbb{y}", "𝕪"),
    ("mathbb{z}", "𝕫"),
    ("mathcal{A}", "𝒜"),
    ("mathcal{B}", "ℬ"),
    ("mathcal{C}", "𝒞"),
    ("mathcal{D}", "𝒟"),
    ("mathcal{E}", "ℰ"),
    ("mathcal{F}", "ℱ"),
    ("mathcal{G}", "𝒢"),
    ("mathcal{H}", "ℋ"),
    ("mathcal{I}", "ℐ"),
    ("mathcal{J}", "𝒥"),
    ("mathcal{K}", "𝒦"),
    ("mathcal{L}", "ℒ"),
    ("mathcal{M}", "ℳ"),
    ("mathcal{N}", "𝒩"),
    ("mathcal{O}", "𝒪"),
    ("mathcal{P}", "𝒫"),
    ("mathcal{Q}", "𝒬"),
    ("mathcal{R}", "ℛ"),
    ("mathcal{S}", "𝒮"),
    ("mathcal{T}", "𝒯"),
    ("mathcal{U}", "𝒰"),
    ("mathcal{V}", "𝒱"),
    ("mathcal{W}", "𝒲"),
    ("mathcal{X}", "𝒳"),
    ("mathcal{Y}", "𝒴"),
    ("mathcal{Z}", "𝒵"),
    ("mathcal{a}", "𝒶"),
    ("mathcal{b}", "𝒷"),
    ("mathcal{c}", "𝒸"),
    ("mathcal{d}", "𝒹"),
    ("mathcal{e}", "ℯ"),
    ("mathcal{f}", "𝒻"),
    ("mathcal{g}", "ℊ"),
    ("mathcal{h}", "𝒽"),
    ("mathcal{i}", "𝒾"),
    ("mathcal{j}", "𝒿"),
    ("mathcal{k}", "𝓀"),
    ("mathcal{l}", "𝓁"),
    ("mathcal{m}", "𝓂"),
    ("mathcal{n}", "𝓃"),
    ("mathcal{o}", "ℴ"),
    ("mathcal{p}", "𝓅"),
    ("mathcal{q}", "𝓆"),
    ("mathcal{r}", "𝓇"),
    ("mathcal{s}", "𝓈"),
    ("mathcal{t}", "𝓉"),
    ("mathcal{u}", "𝓊"),
    ("mathcal{v}", "𝓋"),
    ("mathcal{w}", "𝓌"),
    ("mathcal{x}", "𝓍"),
    ("mathcal{y}", "𝓎"),
    ("mathcal{z}", "𝓏"),
    ("mathfrak{A}", "𝔄"),
    ("mathfrak{B}", "𝔅"),
    ("mathfrak{C}", "ℭ"),
    ("mathfrak{D}", "𝔇"),
    ("mathfrak{E}", "𝔈"),
    ("mathfrak{F}", "𝔉"),
    ("mathfrak{G}", "𝔊"),
    ("mathfrak{H}", "ℌ"),
    ("mathfrak{I}", "ℑ"),
    ("mathfrak{J}", "𝔍"),
    ("mathfrak{K}", "𝔎"),
    ("mathfrak{L}", "𝔏"),
    ("mathfrak{M}", "𝔐"),
    ("mathfrak{N}", "𝔑"),
    ("mathfrak{O}", "𝔒"),
    ("mathfrak{P}", "𝔓"),
    ("mathfrak{Q}", "𝔔"),
    ("mathfrak{R}", "ℜ"),
    ("mathfrak{S}", "𝔖"),
    ("mathfrak{T}", "𝔗"),
    ("mathfrak{U}", "𝔘"),
    ("mathfrak{V}", "𝔙"),
    ("mathfrak{W}", "𝔚"),
    ("mathfrak{X}", "𝔛"),
    ("mathfrak{Y}", "𝔜"),
    ("mathfrak{Z}", "ℨ"),
    ("mathfrak{a}", "𝔞"),
    ("mathfrak{b}", "𝔟"),
    ("mathfrak{c}", "𝔠"),
    ("mathfrak{d}", "𝔡"),
    ("mathfrak{e}", "𝔢"),
    ("mathfrak{f}", "𝔣"),
    ("mathfrak{g}", "𝔤"),
    ("mathfrak{h}", "𝔥"),
    ("mathfrak{i}", "𝔦"),
    ("mathfrak{j}", "𝔧"),
    ("mathfrak{k}", "𝔨"),
    ("mathfrak{l}", "𝔩"),
    ("mathfrak{m}", "𝔪"),
    ("mathfrak{n}", "𝔫"),
    ("mathfrak{o}", "𝔬"),
    ("mathfrak{p}", "𝔭"),
    ("mathfrak{q}", "𝔮"),
    ("mathfrak{r}", "𝔯"),
    ("mathfrak{s}", "𝔰"),
    ("mathfrak{t}", "𝔱"),
    ("mathfrak{u}", "𝔲"),
    ("mathfrak{v}", "𝔳"),
    ("mathfrak{w}", "𝔴"),
    ("mathfrak{x}", "𝔵"),
    ("mathfrak{y}", "𝔶"),
    ("mathfrak{z}", "𝔷"),
    ("mathbf{A}", "𝐀"),
    ("mathbf{B}", "𝐁"),
    ("mathbf{C}", "𝐂"),
    ("mathbf{D}", "𝐃"),
    ("mathbf{E}", "𝐄"),
    ("mathbf{F}", "𝐅"),
    ("mathbf{G}", "𝐆"),
    ("mathbf{H}", "𝐇"),
    ("mathbf{I}", "𝐈"),
    ("mathbf{J}", "𝐉"),
    ("mathbf{K}", "𝐊"),
    ("mathbf{L}", "𝐋"),
    ("mathbf{M}", "𝐌"),
    ("mathbf{N}", "𝐍"),
    ("mathbf{O}", "𝐎"),
    ("mathbf{P}", "𝐏"),
    ("mathbf{Q}", "𝐐"),
    ("mathbf{R}", "𝐑"),
    ("mathbf{S}", "𝐒"),
    ("mathbf{T}", "𝐓"),
    ("mathbf{U}", "𝐔"),
    ("mathbf{V}", "𝐕"),
    ("mathbf{W}", "𝐖"),
    ("mathbf{X}", "𝐗"),
    ("mathbf{Y}", "𝐘"),
    ("mathbf{Z}", "𝐙"),
    ("mathbf{a}", "𝐚"),
    ("mathbf{b}", "𝐛"),
    ("mathbf{c}", "𝐜"),
    ("mathbf{d}", "𝐝"),
    ("mathbf{e}", "𝐞"),
    ("mathbf{f}", "𝐟"),
    ("mathbf{g}", "𝐠"),
    ("mathbf{h}", "𝐡"),
    ("mathbf{i}", "𝐢"),
    ("mathbf{j}", "𝐣"),
    ("mathbf{k}", "𝐤"),
    ("mathbf{l}", "𝐥"),
    ("mathbf{m}", "𝐦"),
    ("mathbf{n}", "𝐧"),
    ("mathbf{o}", "𝐨"),
    ("mathbf{p}", "𝐩"),
    ("mathbf{q}", "𝐪"),
    ("mathbf{r}", "𝐫"),
    ("mathbf{s}", "𝐬"),
    ("mathbf{t}", "𝐭"),
    ("mathbf{u}", "𝐮"),
    ("mathbf{v}", "𝐯"),
    ("mathbf{w}", "𝐰"),
    ("mathbf{x}", "𝐱"),
    ("mathbf{y}", "𝐲"),
    ("mathbf{z}", "𝐳"),
    ("mathit{A}", "𝐴"),
    ("mathit{B}", "𝐵"),
    ("mathit{C}", "𝐶"),
    ("mathit{D}", "𝐷"),
    ("mathit{E}", "𝐸"),
    ("mathit{F}", "𝐹"),
    ("mathit{G}", "𝐺"),
    ("mathit{H}", "𝐻"),
    ("mathit{I}", "𝐼"),
    ("mathit{J}", "𝐽"),
    ("mathit{K}", "𝐾"),
    ("mathit{L}", "𝐿"),
    ("mathit{M}", "𝑀"),
    ("mathit{N}", "𝑁"),
    ("mathit{O}", "𝑂"),
    ("mathit{P}", "𝑃"),
    ("mathit{Q}", "𝑄"),
    ("mathit{R}", "𝑅"),
    ("mathit{S}", "𝑆"),
    ("mathit{T}", "𝑇"),
    ("mathit{U}", "𝑈"),
    ("mathit{V}", "𝑉"),
    ("mathit{W}", "𝑊"),
    ("mathit{X}", "𝑋"),
    ("mathit{Y}", "𝑌"),
    ("mathit{Z}", "𝑍"),
    ("mathit{a}", "𝑎"),
    ("mathit{b}", "𝑏"),
    ("mathit{c}", "𝑐"),
    ("mathit{d}", "𝑑"),
    ("mathit{e}", "𝑒"),
    ("mathit{f}", "𝑓"),
    ("mathit{g}", "𝑔"),
    ("mathit{h}", "ℎ"),
    ("mathit{i}", "𝑖"),
    ("mathit{j}", "𝑗"),
    ("mathit{k}", "𝑘"),
    ("mathit{l}", "𝑙"),
    ("mathit{m}", "𝑚"),
    ("mathit{n}", "𝑛"),
    ("mathit{o}", "𝑜"),
    ("mathit{p}", "𝑝"),
    ("mathit{q}", "𝑞"),
    ("mathit{r}", "𝑟"),
    ("mathit{s}", "𝑠"),
    ("mathit{t}", "𝑡"),
    ("mathit{u}", "𝑢"),
    ("mathit{v}", "𝑣"),
    ("mathit{w}", "𝑤"),
    ("mathit{x}", "𝑥"),
    ("mathit{y}", "𝑦"),
    ("mathit{z}", "𝑧"),
    ("mathsf{A}", "𝖠"),
    ("mathsf{B}", "𝖡"),
    ("mathsf{C}", "𝖢"),
    ("mathsf{D}", "𝖣"),
    ("mathsf{E}", "𝖤"),
    ("mathsf{F}", "𝖥"),
    ("mathsf{G}", "𝖦"),
    ("mathsf{H}", "𝖧"),
    ("mathsf{I}", "𝖨"),
    ("mathsf{J}", "𝖩"),
    ("mathsf{K}", "𝖪"),
    ("mathsf{L}", "𝖫"),
    ("mathsf{M}", "𝖬"),
    ("mathsf{N}", "𝖭"),
    ("mathsf{O}", "𝖮"),
    ("mathsf{P}", "𝖯"),
    ("mathsf{Q}", "𝖰"),
    ("mathsf{R}", "𝖱"),
    ("mathsf{S}", "𝖲"),
    ("mathsf{T}", "𝖳"),
    ("mathsf{U}", "𝖴"),
    ("mathsf{V}", "𝖵"),
    ("mathsf{W}", "𝖶"),
    ("mathsf{X}", "𝖷"),
    ("mathsf{Y}", "𝖸"),
    ("mathsf{Z}", "𝖹"),
    ("mathsf{a}", "𝖺"),
    ("mathsf{b}", "𝖻"),
    ("mathsf{c}", "𝖼"),
    ("mathsf{d}", "𝖽"),
    ("mathsf{e}", "𝖾"),
    ("mathsf{f}", "𝖿"),
    ("mathsf{g}", "𝗀"),
    ("mathsf{h}", "𝗁"),
    ("mathsf{i}", "𝗂"),
    ("mathsf{j}", "𝗃"),
    ("mathsf{k}", "𝗄"),
    ("mathsf{l}", "𝗅"),
    ("mathsf{m}", "𝗆"),
    ("mathsf{n}", "𝗇"),
    ("mathsf{o}", "𝗈"),
    ("mathsf{p}", "𝗉"),
    ("mathsf{q}", "𝗊"),
    ("mathsf{r}", "𝗋"),
    ("mathsf{s}", "𝗌"),
    ("mathsf{t}", "𝗍"),
    ("mathsf{u}", "𝗎"),
    ("mathsf{v}", "𝗏"),
    ("mathsf{w}", "𝗐"),
    ("mathsf{x}", "𝗑"),
    ("mathsf{y}", "𝗒"),
    ("mathsf{z}", "𝗓"),
    ("mathtt{A}", "𝙰"),
    ("mathtt{B}", "𝙱"),
    ("mathtt{C}", "𝙲"),
    ("mathtt{D}", "𝙳"),
    ("mathtt{E}", "𝙴"),
    ("mathtt{F}", "𝙵"),
    ("mathtt{G}", "𝙶"),
    ("mathtt{H}", "𝙷"),
    ("mathtt{I}", "𝙸"),
    ("mathtt{J}", "𝙹"),
    ("mathtt{K}", "𝙺"),
    ("mathtt{L}", "𝙻"),
    ("mathtt{M}", "𝙼"),
    ("mathtt{N}", "𝙽"),
    ("mathtt{O}", "𝙾"),
    ("mathtt{P}", "𝙿"),
    ("mathtt{Q}", "𝚀"),
    ("mathtt{R}", "𝚁"),
    ("mathtt{S}", "𝚂"),
    ("mathtt{T}", "𝚃"),
    ("mathtt{U}", "𝚄"),
    ("mathtt{V}", "𝚅"),
    ("mathtt{W}", "𝚆"),
    ("mathtt{X}", "𝚇"),
    ("mathtt{Y}", "𝚈"),
    ("mathtt{Z}", "𝚉"),
    ("mathtt{a}", "𝚊"),
    ("mathtt{b}", "𝚋"),
    ("mathtt{c}", "𝚌"),
    ("mathtt{d}", "𝚍"),
    ("mathtt{e}", "𝚎"),
    ("mathtt{f}", "𝚏"),
    ("mathtt{g}", "𝚐"),
    ("mathtt{h}", "𝚑"),
    ("mathtt{i}", "𝚒"),
    ("mathtt{j}", "𝚓"),
    ("mathtt{k}", "𝚔"),
    ("mathtt{l}", "𝚕"),
    ("mathtt{m}", "𝚖"),
    ("mathtt{n}", "𝚗"),
    ("mathtt{o}", "𝚘"),
    ("mathtt{p}", "𝚙"),
    ("mathtt{q}", "𝚚"),
    ("mathtt{r}", "𝚛"),
    ("mathtt{s}", "𝚜"),
    ("mathtt{t}", "𝚝"),
    ("mathtt{u}", "𝚞"),
    ("mathtt{v}", "𝚟"),
    ("mathtt{w}", "𝚠"),
    ("mathtt{x}", "𝚡"),
    ("mathtt{y}", "𝚢"),
    ("mathtt{z}", "𝚣"),
];
pub(crate) const LARGE_OPERATORS: &[(&str, char)] = &[
    ("sum", '∑'),
    ("prod", '∏'),
    ("int", '∫'),
    ("oint", '∮'),
    ("iint", '∬'),
    ("iiint", '∭'),
    ("bigcup", '⋃'),
    ("bigcap", '⋂'),
    ("coprod", '∐'),
    ("bigoplus", '⨁'),
    ("bigotimes", '⨂'),
];
