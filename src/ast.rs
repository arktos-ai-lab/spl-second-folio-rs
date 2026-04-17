/// Abstract Syntax Tree for the Shakespeare Programming Language.

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Expr {
    /// "I" / "me" / "myself" — the speaking character's own value
    FirstPerson,
    /// "you" / "thee" / "yourself" — the character being spoken to
    SecondPerson,
    /// A reference to another character by name (e.g. "Romeo" in an expression)
    CharRef(String),
    /// A positive noun phrase: value = 2^(adjective_count)
    Positive { adjective_count: u32 },
    /// A negative noun phrase: value = -(2^(adjective_count))
    Negative { adjective_count: u32 },
    /// "nothing" — literal 0
    Nothing,
    /// Unary arithmetic
    Unary { op: UnaryOp, operand: Box<Expr> },
    /// Binary arithmetic
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
}

impl Expr {
    pub fn positive(adj: u32) -> Self { Expr::Positive { adjective_count: adj } }
    pub fn negative(adj: u32) -> Self { Expr::Negative { adjective_count: adj } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Square,       // "the square of"
    Cube,         // "the cube of"
    SquareRoot,   // "the square root of"
    Factorial,    // "the factorial of"
    Twice,        // "twice"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Sum,          // "the sum of X and Y"
    Difference,   // "the difference between X and Y"
    Product,      // "the product of X and Y"
    Quotient,     // "the quotient between X and Y"
    Remainder,    // "the remainder of the quotient between X and Y"
}

// ---------------------------------------------------------------------------
// Comparisons (used in Question sentences)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Equal,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    NotEqual,
}

#[derive(Debug, Clone)]
pub struct Comparison {
    pub op: CmpOp,
    pub left: Expr,
    pub right: Expr,
}

// ---------------------------------------------------------------------------
// I/O kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoKind { Numeric, Char }

// ---------------------------------------------------------------------------
// Conditional prefix
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    IfSo,   // execute if global_boolean == true
    IfNot,  // execute if global_boolean == false
}

// ---------------------------------------------------------------------------
// Speech mode (SPL-F2 staging extension)
// ---------------------------------------------------------------------------

/// Describes how a dialogue block is staged in SPL-F2.
/// Emitted as `Op::BeginSpeech` at each dialogue header when parsing in F2 mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeechMode {
    /// SPL 1.0 default: implicit second person — requires exactly 2 on stage.
    Normal,
    /// `CHAR to ADDRESSEE:` — explicit addressee binding; any stage count allowed.
    To(String),
    /// `CHAR aside:` — aside block; second-person operations forbidden.
    Aside,
    /// `CHAR aside to ADDRESSEE:` — directed aside; computationally binds addressee.
    AsideTo(String),
    /// `CHAR solus:` — speaker must be alone on stage; second-person ops forbidden.
    Solus,
}

// ---------------------------------------------------------------------------
// Operations (flattened — speaker is tracked separately)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Op {
    /// [Enter CharA, CharB, ...]
    Enter(Vec<String>),
    /// [Exit CharA]
    Exit(String),
    /// [Exeunt CharA, CharB] or [Exeunt] (all)
    Exeunt(Vec<String>),   // empty vec = all

    /// Assignment: CHAR You are EXPR!
    Assign {
        expr: Expr,
        cond: Option<Cond>,
    },
    /// Output the speaking-target's value
    Output { kind: IoKind, cond: Option<Cond> },
    /// Input into the speaking-target's register
    Input  { kind: IoKind, cond: Option<Cond> },
    /// Push EXPR onto the speaking-target's stack
    Push { expr: Expr, cond: Option<Cond> },
    /// Pop the speaking-target's stack into their own value
    Pop  { cond: Option<Cond> },
    /// Question — sets global_boolean
    Question { cmp: Comparison },
    /// Goto a scene
    Goto {
        act: u32,
        scene: u32,
        cond: Option<Cond>,
    },
    /// SPL-F2 ornamental sentence — a non-operational literary no-op.
    /// Has no effect on machine state; accepted only when the sentence
    /// does not begin with a reserved operational head and contains no
    /// incomplete executable skeleton (Rules O2–O5 from the F2 spec).
    Ornament,

    /// SPL-F2 staging marker: emitted once per dialogue header in F2 mode.
    /// Updates the interpreter's speech mode and validates staging constraints
    /// (solus stage count, explicit addressee on stage) at block entry.
    BeginSpeech(SpeechMode),
}

// ---------------------------------------------------------------------------
// Play structure
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CharacterDecl {
    pub name: String,
    pub description: String,
}

/// A flattened operation with its speaker and source line.
#[derive(Debug, Clone)]
pub struct FlatOp {
    /// The character currently speaking (owns the operation).
    /// For stage directions, speaker is empty string.
    pub speaker: String,
    /// The operation itself.
    pub op: Op,
    /// Source line number (1-based) for error messages.
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Play {
    pub title: String,
    pub characters: Vec<CharacterDecl>,
    /// All operations in execution order (acts/scenes flattened).
    pub ops: Vec<FlatOp>,
    /// Map of (act, scene) -> index into ops for Goto resolution.
    pub scene_index: std::collections::HashMap<(u32, u32), usize>,
    /// Attestation warnings produced by F1-profile parsing. Empty in Classic/F2 mode.
    pub warnings: Vec<String>,
}
