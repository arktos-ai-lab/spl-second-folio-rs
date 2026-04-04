/// SPL parser — recursive descent over tokenised lines.
///
/// Parsing strategy:
///   1. Split source into lines, strip punctuation at end of each sentence.
///   2. Identify sections by structure (title, dramatis personae, acts/scenes,
///      stage directions, dialogue).
///   3. Parse each sentence by matching keyword patterns (case-insensitive).

use std::collections::HashMap;

use crate::ast::*;
use crate::error::{Result, SplError};
use crate::vocab::{
    as_character_name, is_article, is_negative_adjective, is_negative_noun,
    is_positive_adjective, is_positive_noun, parse_roman,
};

// ---------------------------------------------------------------------------
// Token helpers
// ---------------------------------------------------------------------------

/// Normalise a word: lowercase, strip leading/trailing punctuation
/// but keep internal punctuation (e.g. apostrophes).
fn norm(s: &str) -> String {
    s.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn words(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

// ---------------------------------------------------------------------------
// Expression parser
// ---------------------------------------------------------------------------

struct ExprParser<'a> {
    tokens: &'a [&'a str],
    pos: usize,
    line: usize,
    /// Declared character names (lowercase) for this play.
    char_names: &'a [String],
}

impl<'a> ExprParser<'a> {
    fn new(tokens: &'a [&'a str], line: usize, char_names: &'a [String]) -> Self {
        ExprParser { tokens, pos: 0, line, char_names }
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).copied()
    }

    fn peek_norm(&self) -> Option<String> {
        self.peek().map(|s| norm(s))
    }

    fn consume(&mut self) -> Option<&str> {
        let t = self.tokens.get(self.pos).copied();
        if t.is_some() { self.pos += 1; }
        t
    }

    fn expect_word(&mut self, expected: &str) -> Result<()> {
        match self.peek_norm().as_deref() {
            Some(w) if w == expected => { self.consume(); Ok(()) }
            other => Err(SplError::parse(
                self.line,
                format!("expected '{}', got {:?}", expected, other),
            )),
        }
    }

    fn expect_any(&mut self, words: &[&str]) -> Result<String> {
        if let Some(w) = self.peek_norm() {
            if words.iter().any(|&e| e == w) {
                self.consume();
                return Ok(w);
            }
        }
        Err(SplError::parse(
            self.line,
            format!("expected one of {:?}", words),
        ))
    }

    /// Try to consume "and" between operands.
    fn maybe_and(&mut self) {
        if self.peek_norm().as_deref() == Some("and") {
            self.consume();
        }
    }

    /// Try to consume a character name (greedy, longest match) from current position.
    /// Only matches names that are actually declared in this play.
    fn try_consume_char_name(&mut self) -> Option<String> {
        let remaining = self.tokens.len() - self.pos;
        let max_len = remaining.min(4);
        let mut best: Option<(usize, String)> = None;
        for len in 1..=max_len {
            let words: Vec<String> = self.tokens[self.pos..self.pos + len]
                .iter().map(|s| norm(s)).collect();
            let joined = words.join(" ");
            // Only accept if declared in this play
            if self.char_names.iter().any(|n| n == &joined) {
                best = Some((len, joined));
            }
        }
        if let Some((len, name)) = best {
            self.pos += len;
            Some(name)
        } else {
            None
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr> {
        // Skip articles at start — but NOT "the", which is also the prefix for
        // operator forms like "the sum of X and Y".  We let "the" fall through to
        // the dedicated match arm below so parse_the_expr() can decide whether it's
        // an operator or just "the horse" (a noun phrase with "the" as article).
        while let Some(w) = self.peek_norm().as_deref() {
            if w != "the" && is_article(w) {
                self.consume();
            } else {
                break;
            }
        }

        match self.peek_norm().as_deref() {
            // First person pronouns
            Some("i") | Some("me") | Some("myself") => {
                self.consume();
                Ok(Expr::FirstPerson)
            }
            // Second person pronouns
            Some("you") | Some("thee") | Some("thou") | Some("yourself")
            | Some("thyself") => {
                self.consume();
                Ok(Expr::SecondPerson)
            }
            Some("nothing") => {
                self.consume();
                Ok(Expr::Nothing)
            }
            // Binary operations
            Some("the") => self.parse_the_expr(),
            // Unary "twice"
            Some("twice") => {
                self.consume();
                let inner = self.parse_expr()?;
                Ok(Expr::Unary { op: UnaryOp::Twice, operand: Box::new(inner) })
            }
            _ => {
                // Try character name reference before noun phrase
                if let Some(name) = self.try_consume_char_name() {
                    return Ok(Expr::CharRef(name));
                }
                self.parse_noun_phrase()
            }
        }
    }

    fn parse_the_expr(&mut self) -> Result<Expr> {
        self.expect_word("the")?;
        match self.peek_norm().as_deref() {
            Some("sum") => {
                self.consume();
                self.expect_word("of")?;
                let l = self.parse_expr()?;
                self.maybe_and();
                let r = self.parse_expr()?;
                Ok(Expr::Binary { op: BinaryOp::Sum, left: Box::new(l), right: Box::new(r) })
            }
            Some("difference") => {
                self.consume();
                self.expect_word("between")?;
                let l = self.parse_expr()?;
                self.maybe_and();
                let r = self.parse_expr()?;
                Ok(Expr::Binary { op: BinaryOp::Difference, left: Box::new(l), right: Box::new(r) })
            }
            Some("product") => {
                self.consume();
                self.expect_word("of")?;
                let l = self.parse_expr()?;
                self.maybe_and();
                let r = self.parse_expr()?;
                Ok(Expr::Binary { op: BinaryOp::Product, left: Box::new(l), right: Box::new(r) })
            }
            Some("quotient") => {
                self.consume();
                self.expect_word("between")?;
                let l = self.parse_expr()?;
                self.maybe_and();
                let r = self.parse_expr()?;
                Ok(Expr::Binary { op: BinaryOp::Quotient, left: Box::new(l), right: Box::new(r) })
            }
            Some("remainder") => {
                self.consume();
                // "remainder of the quotient between X and Y"
                self.expect_word("of")?;
                self.expect_word("the")?;
                self.expect_word("quotient")?;
                self.expect_word("between")?;
                let l = self.parse_expr()?;
                self.maybe_and();
                let r = self.parse_expr()?;
                Ok(Expr::Binary { op: BinaryOp::Remainder, left: Box::new(l), right: Box::new(r) })
            }
            // "the twice X" — treat "the" as decorative before "twice"
            Some("twice") => {
                self.consume();
                let inner = self.parse_expr()?;
                Ok(Expr::Unary { op: UnaryOp::Twice, operand: Box::new(inner) })
            }
            Some("square") => {
                self.consume();
                match self.peek_norm().as_deref() {
                    Some("root") => {
                        self.consume();
                        self.expect_word("of")?;
                        let inner = self.parse_expr()?;
                        Ok(Expr::Unary { op: UnaryOp::SquareRoot, operand: Box::new(inner) })
                    }
                    Some("of") => {
                        self.consume();
                        let inner = self.parse_expr()?;
                        Ok(Expr::Unary { op: UnaryOp::Square, operand: Box::new(inner) })
                    }
                    _ => Err(SplError::parse(self.line, "expected 'root' or 'of' after 'square'")),
                }
            }
            Some("cube") => {
                self.consume();
                self.expect_word("of")?;
                let inner = self.parse_expr()?;
                Ok(Expr::Unary { op: UnaryOp::Cube, operand: Box::new(inner) })
            }
            Some("factorial") => {
                self.consume();
                self.expect_word("of")?;
                let inner = self.parse_expr()?;
                Ok(Expr::Unary { op: UnaryOp::Factorial, operand: Box::new(inner) })
            }
            _ => {
                // Check if "the [remaining...]" forms a character name (e.g. "the Ghost").
                // "the" was already consumed; try to match remaining tokens with "the " prepended.
                let saved_pos = self.pos;
                for len in (1..=self.tokens.len().saturating_sub(self.pos).min(4)).rev() {
                    let words: Vec<String> = self.tokens[self.pos..self.pos + len]
                        .iter().map(|s| norm(s)).collect();
                    let with_the = format!("the {}", words.join(" "));
                    if self.char_names.iter().any(|n| n == &with_the) {
                        self.pos += len;
                        return Ok(Expr::CharRef(with_the));
                    }
                }
                self.pos = saved_pos;
                // "the" used as a noun-phrase article: "the horse", "the beautiful rose", etc.
                // We already consumed "the"; parse_noun_phrase skips any remaining articles.
                self.parse_noun_phrase()
            }
        }
    }

    /// Parse a noun phrase: (article?) (adjective*) noun
    ///
    /// Sign rule: always determined by the noun, regardless of adjectives.
    /// All adjectives (positive or negative) double the magnitude.
    fn parse_noun_phrase(&mut self) -> Result<Expr> {
        // Skip articles
        while self.peek_norm().as_deref().map(|w| is_article(w)).unwrap_or(false) {
            self.consume();
        }

        let mut adj_count: u32 = 0;

        // Consume adjectives (all adjectives just double magnitude)
        loop {
            match self.peek_norm().as_deref() {
                Some(w) if is_positive_adjective(w) || is_negative_adjective(w) => {
                    adj_count += 1;
                    self.consume();
                }
                _ => break,
            }
            // Skip articles between adjectives
            while self.peek_norm().as_deref().map(|w| is_article(w)).unwrap_or(false) {
                self.consume();
            }
        }

        // Consume the noun — noun always determines sign
        match self.peek_norm().as_deref() {
            Some(w) if is_positive_noun(w) => {
                self.consume();
                Ok(Expr::positive(adj_count))
            }
            Some(w) if is_negative_noun(w) => {
                self.consume();
                Ok(Expr::negative(adj_count))
            }
            other => Err(SplError::parse(
                self.line,
                format!("expected a noun in noun phrase, got {:?}", other),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Comparison parser helper (used in Question sentences)
// ---------------------------------------------------------------------------

fn parse_comparison(tokens: &[&str], line: usize, char_names: &[String]) -> Result<Comparison> {
    // Patterns:
    //   "Am I as good as X?"           → first == X (numeric "as good as" = equal)
    //   "Is X greater than Y?"
    //   "Is X less than Y?"
    //   "Is X as good as Y?" / "Are X and Y equal?"
    //   "Is X not as good as Y?"
    //
    // We do a best-effort keyword scan.
    let toks: Vec<String> = tokens.iter().map(|s| norm(s)).collect();
    let ts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();

    // Find "as X as" (any single word between two "as") — means Equal in comparisons.
    // We search BEFORE other patterns so "not as good as" is still checked below.
    let as_x_as_pos: Option<usize> = if !contains_seq(&ts, &["not", "as", "good", "as"]) {
        (0..ts.len().saturating_sub(2))
            .find(|&i| ts[i] == "as" && ts[i+2] == "as")
    } else {
        None
    };

    let (cmp_op, op_start, op_len): (CmpOp, usize, usize) =
    if contains_seq(&ts, &["not", "as", "good", "as"]) {
        (CmpOp::NotEqual, find_seq(&ts, &["not", "as", "good", "as"]).unwrap(), 4)
    } else if contains_seq(&ts, &["equal"]) {
        (CmpOp::Equal, find_seq(&ts, &["equal"]).unwrap(), 1)
    } else if let Some(pos) = as_x_as_pos {
        (CmpOp::Equal, pos, 3)
    } else if let Some(pos) = find_seq(&ts, &["greater", "than", "or", "equal"])
        .or_else(|| find_seq(&ts, &["better", "than", "or", "equal"])) {
        (CmpOp::GreaterOrEqual, pos, 4)
    } else if let Some(pos) = find_seq(&ts, &["less", "than", "or", "equal"])
        .or_else(|| find_seq(&ts, &["worse", "than", "or", "equal"])) {
        (CmpOp::LessOrEqual, pos, 4)
    } else if let Some(pos) = find_seq(&ts, &["greater", "than"])
        .or_else(|| find_seq(&ts, &["better", "than"]))
        .or_else(|| find_seq(&ts, &["more", "cunning", "than"]).map(|p| (p, 3usize)).map(|(p,l)| p))
        .or_else(|| find_seq(&ts, &["more", "than"]))
        .or_else(|| find_seq(&ts, &["bigger", "than"]))
        .or_else(|| find_seq(&ts, &["larger", "than"]))
        .or_else(|| find_seq(&ts, &["older", "than"]))
    {
        let len = if find_seq(&ts, &["more", "cunning", "than"]) == Some(pos) { 3 } else { 2 };
        (CmpOp::GreaterThan, pos, len)
    } else if let Some(pos) = find_seq(&ts, &["less", "than"])
        .or_else(|| find_seq(&ts, &["worse", "than"]))
        .or_else(|| find_seq(&ts, &["smaller", "than"]))
        .or_else(|| find_seq(&ts, &["younger", "than"]))
        .or_else(|| find_seq(&ts, &["shorter", "than"]))
    {
        (CmpOp::LessThan, pos, 2)
    } else {
        return Err(SplError::parse(line, "unrecognised comparison operator"));
    };

    // Split tokens around the operator sequence
    let left_tokens = &tokens[..op_start];
    let right_tokens = &tokens[op_start + op_len..];

    // Strip leading "is" / "am" / "are" / "art" / "?" from left
    let left_stripped = strip_question_lead(left_tokens);
    let right_stripped = strip_question_trail(right_tokens);

    let left  = ExprParser::new(left_stripped, line, char_names).parse_expr()?;
    let right = ExprParser::new(right_stripped, line, char_names).parse_expr()?;
    Ok(Comparison { op: cmp_op, left, right })
}

fn contains_seq(haystack: &[&str], needle: &[&str]) -> bool {
    find_seq(haystack, needle).is_some()
}

fn find_seq(haystack: &[&str], needle: &[&str]) -> Option<usize> {
    if needle.is_empty() { return Some(0); }
    if needle.len() > haystack.len() { return None; }
    'outer: for i in 0..=haystack.len().saturating_sub(needle.len()) {
        for (j, n) in needle.iter().enumerate() {
            if haystack[i + j] != *n { continue 'outer; }
        }
        return Some(i);
    }
    None
}

fn strip_question_lead<'a>(tokens: &'a [&'a str]) -> &'a [&'a str] {
    let mut start = 0;
    let skip = ["is", "am", "are", "art", "?"];
    while start < tokens.len() {
        let w = norm(tokens[start]);
        if skip.iter().any(|s| *s == w.as_str()) {
            start += 1;
        } else {
            break;
        }
    }
    &tokens[start..]
}

fn strip_question_trail<'a>(tokens: &'a [&'a str]) -> &'a [&'a str] {
    let mut end = tokens.len();
    while end > 0 {
        let w = norm(tokens[end - 1]);
        if w == "?" || w.is_empty() { end -= 1; } else { break; }
    }
    &tokens[..end]
}

// ---------------------------------------------------------------------------
// Sentence parser
// ---------------------------------------------------------------------------

fn parse_sentence(tokens: &[&str], line: usize, char_names: &[String]) -> Result<Op> {
    if tokens.is_empty() {
        return Err(SplError::parse(line, "empty sentence"));
    }

    let toks: Vec<String> = tokens.iter().map(|s| norm(s)).collect();
    let ts: Vec<&str> = toks.iter().map(|s| s.as_str()).collect();

    // Detect conditional prefix
    let (cond, rest_tokens) = if ts.first() == Some(&"if") {
        match ts.get(1).copied() {
            Some("so") => (Some(Cond::IfSo), &tokens[2..]),
            Some("not") => (Some(Cond::IfNot), &tokens[2..]),
            _ => (None, tokens),
        }
    } else {
        (None, tokens)
    };

    let rt: Vec<String> = rest_tokens.iter().map(|s| norm(s)).collect();
    let rs: Vec<&str> = rt.iter().map(|s| s.as_str()).collect();

    // Question — must check before assignment
    // "Am I...", "Is X...", "Are X...", "Art thou..."
    let is_question_lead = matches!(rs.first().copied(), Some("am") | Some("is") | Some("are") | Some("art"));
    if is_question_lead {
        if let Ok(cmp) = parse_comparison(rest_tokens, line, char_names) {
            return Ok(Op::Question { cmp });
        }
    }

    // Goto: "let us proceed to scene N" / "we must return to scene N" / etc.
    if rs.first() == Some(&"let") || rs.first() == Some(&"we") {
        if let Ok(op) = parse_goto(&rs, line, cond) {
            return Ok(op);
        }
    }

    // Output: "speak your/thy mind" (number) / "open your/thy heart" (char)
    if contains_seq(&rs, &["speak", "your", "mind"])
        || contains_seq(&rs, &["speak", "thy", "mind"])
        || contains_seq(&rs, &["open", "your", "heart"])
        || contains_seq(&rs, &["open", "thy", "heart"])
    {
        return Ok(Op::Output { kind: IoKind::Numeric, cond });
    }
    if contains_seq(&rs, &["speak", "your", "heart"])
        || contains_seq(&rs, &["speak", "thy", "heart"])
        || contains_seq(&rs, &["open", "your", "mouth"])
        || contains_seq(&rs, &["open", "thy", "mouth"])
    {
        return Ok(Op::Output { kind: IoKind::Char, cond });
    }

    // Input: "listen to your heart" (number) / "open your mind" (char)
    if contains_seq(&rs, &["listen", "to", "your", "heart"]) {
        return Ok(Op::Input { kind: IoKind::Numeric, cond });
    }
    if contains_seq(&rs, &["listen", "to", "your", "mind"])
        || contains_seq(&rs, &["open", "your", "mind"])
    {
        return Ok(Op::Input { kind: IoKind::Char, cond });
    }

    // Push: "remember ..." / "remember yourself"
    if rs.first() == Some(&"remember") {
        let expr_tokens = &rest_tokens[1..];
        if expr_tokens.is_empty()
            || norm(expr_tokens[0]) == "yourself"
            || norm(expr_tokens[0]) == "thyself"
        {
            return Ok(Op::Push { expr: Expr::SecondPerson, cond });
        }
        let expr = ExprParser::new(expr_tokens, line, char_names).parse_expr()?;
        return Ok(Op::Push { expr, cond });
    }

    // Pop: "recall ..."
    if rs.first() == Some(&"recall") {
        return Ok(Op::Pop { cond });
    }

    // Assignment: "you are ..." / "thou art ..."
    if (rs.first() == Some(&"you") && rs.get(1) == Some(&"are"))
        || (rs.first() == Some(&"thou") && rs.get(1) == Some(&"art"))
    {
        let mut expr_tokens = &rest_tokens[2..];
        // Strip decorative "as [word] as" comparative prefix (e.g. "you are as brave as X")
        if expr_tokens.len() >= 3 && norm(expr_tokens[0]) == "as" && norm(expr_tokens[2]) == "as" {
            expr_tokens = &expr_tokens[3..];
        }
        let expr = ExprParser::new(expr_tokens, line, char_names).parse_expr()?;
        return Ok(Op::Assign { expr, cond });
    }

    // Short assignment: "you <noun-phrase>" / "thou <noun-phrase>" (no copula)
    // e.g. "You lying stupid coward!" — second word is an adjective or noun, not a verb.
    if rs.first() == Some(&"you") || rs.first() == Some(&"thou") {
        if let Some(w) = rs.get(1) {
            let is_verb = matches!(*w, "are" | "art" | "is" | "am" | "were" | "was");
            if !is_verb {
                let expr_tokens = &rest_tokens[1..];
                if let Ok(expr) = ExprParser::new(expr_tokens, line, char_names).parse_expr() {
                    return Ok(Op::Assign { expr, cond });
                }
            }
        }
    }

    Err(SplError::parse(line, format!("unrecognised sentence: {:?}", &ts[..ts.len().min(6)])))
}

fn parse_goto(rs: &[&str], line: usize, cond: Option<Cond>) -> Result<Op> {
    // "let us proceed to act N" / "let us proceed to scene N"
    // "let us return to scene N"
    if let Some(to_pos) = rs.iter().position(|&w| w == "to") {
        let rest = &rs[to_pos + 1..];
        let (act_num, scene_num) = match rest.first().copied() {
            Some("scene") => {
                let scene = rest.get(1)
                    .and_then(|w| parse_roman(w))
                    .ok_or_else(|| SplError::parse(line, "invalid scene number"))?;
                (1, scene) // no act specified → current act (we use 1 as placeholder)
            }
            Some("act") => {
                let act = rest.get(1)
                    .and_then(|w| parse_roman(w))
                    .ok_or_else(|| SplError::parse(line, "invalid act number"))?;
                let scene = if rest.get(2) == Some(&"scene") {
                    rest.get(3)
                        .and_then(|w| parse_roman(w))
                        .unwrap_or(1)
                } else {
                    1
                };
                (act, scene)
            }
            _ => return Err(SplError::parse(line, "expected 'act' or 'scene' after 'to'")),
        };
        Ok(Op::Goto { act: act_num, scene: scene_num, cond })
    } else {
        Err(SplError::parse(line, "malformed goto: missing 'to'"))
    }
}

// ---------------------------------------------------------------------------
// Stage direction parser
// ---------------------------------------------------------------------------

fn parse_stage_direction(inner: &str, line: usize) -> Result<Op> {
    let tokens: Vec<&str> = inner.trim().split_whitespace().collect();
    if tokens.is_empty() {
        return Err(SplError::parse(line, "empty stage direction"));
    }
    let first = norm(tokens[0]);
    match first.as_str() {
        "enter" => {
            let names = parse_character_list(&tokens[1..], line)?;
            Ok(Op::Enter(names))
        }
        "exit" => {
            let names = parse_character_list(&tokens[1..], line)?;
            if names.len() != 1 {
                return Err(SplError::parse(line, "Exit expects exactly one character"));
            }
            Ok(Op::Exit(names.into_iter().next().unwrap()))
        }
        "exeunt" => {
            let names = parse_character_list(&tokens[1..], line)?;
            Ok(Op::Exeunt(names))
        }
        _ => Err(SplError::parse(line, format!("unknown stage direction: {:?}", first))),
    }
}

/// Parse a list of (possibly multi-word) character names separated by commas and/or "and".
///
/// Stage directions use both forms:
///   [Enter Hamlet and Ophelia]
///   [Exeunt Hamlet, Ophelia, and Horatio]
///
/// Multi-word names (e.g. "Don Pedro", "Friar Laurence") require greedy longest-first
/// matching: we try to consume as many tokens as possible before hitting a separator.
fn parse_character_list(tokens: &[&str], line: usize) -> Result<Vec<String>> {
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    // Normalise: strip trailing commas from each token
    let clean: Vec<String> = tokens
        .iter()
        .map(|t| t.trim_matches(|c: char| c == ',').to_string())
        .filter(|t| !t.is_empty())
        .collect();

    let mut names = Vec::new();
    let mut i = 0;

    while i < clean.len() {
        // Skip "and" separators
        if clean[i].to_lowercase() == "and" {
            i += 1;
            continue;
        }

        // Greedy: try the longest possible multi-word match first.
        // Build candidates from longest (remaining tokens) down to 1.
        let max_len = clean.len() - i;
        let mut matched = false;
        for len in (1..=max_len).rev() {
            // Skip if any token in this window is "and"
            if clean[i..i + len].iter().any(|t| t.to_lowercase() == "and") {
                continue;
            }
            let candidate = clean[i..i + len].join(" ");
            if let Some(n) = as_character_name(&candidate) {
                names.push(n.to_string());
                i += len;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(SplError::parse(
                line,
                format!("unrecognised character name starting at: {:?}", clean[i]),
            ));
        }
    }
    Ok(names)
}

// ---------------------------------------------------------------------------
// Top-level play parser
// ---------------------------------------------------------------------------

pub fn parse_play(source: &str) -> Result<Play> {
    let mut lines: Vec<&str> = source.lines().collect();
    let mut line_no = 0usize;

    // Helper: advance past blank lines
    let skip_blank = |idx: &mut usize, lines: &[&str]| {
        while *idx < lines.len() && lines[*idx].trim().is_empty() {
            *idx += 1;
        }
    };

    skip_blank(&mut line_no, &lines);

    // --- Title ---
    let title = if line_no < lines.len() {
        let t = lines[line_no].trim().trim_end_matches('.').to_string();
        line_no += 1;
        t
    } else {
        return Err(SplError::parse(0, "play has no title"));
    };

    // --- Dramatis Personae ---
    // Continues until we hit a blank line followed by "Act"
    skip_blank(&mut line_no, &lines);
    let mut characters = Vec::new();
    while line_no < lines.len() {
        let l = lines[line_no].trim();
        if l.is_empty() { line_no += 1; continue; }
        let lower = l.to_lowercase();
        if lower.starts_with("act ") { break; }

        // Character declaration: "Name, description."
        if let Some(comma) = l.find(',') {
            let name_part = l[..comma].trim();
            let desc_part = l[comma + 1..].trim().trim_end_matches('.');
            match as_character_name(name_part) {
                Some(canonical) => {
                    characters.push(CharacterDecl {
                        name: canonical.to_string(),
                        description: desc_part.to_string(),
                    });
                }
                None => {} // silently skip non-character lines in this section
            }
        }
        line_no += 1;
    }

    // Collect declared character names (lowercase) for use during expression parsing.
    let char_names: Vec<String> = characters.iter().map(|c| c.name.clone()).collect();

    // --- Acts and Scenes ---
    let mut ops: Vec<FlatOp> = Vec::new();
    let mut scene_index: HashMap<(u32, u32), usize> = HashMap::new();
    let mut current_act = 1u32;
    let mut current_scene = 1u32;
    let mut current_speaker = String::new();
    // Buffer to accumulate dialogue across physical lines (sentences may span lines).
    let mut pending_dialogue = String::new();
    let mut pending_line = 0usize;

    // Drain all complete sentences (ending with . ! ?) from pending_dialogue.
    let flush_pending = |pending: &mut String, speaker: &str, pline: usize, ops: &mut Vec<FlatOp>, cnames: &[String]| {
        loop {
            let term = pending.find(|c| c == '.' || c == '!' || c == '?');
            match term {
                None => break,
                Some(pos) => {
                    let end = pos + 1;
                    let sentence_str = pending[..end].trim().to_string();
                    *pending = pending[end..].trim_start().to_string();
                    if sentence_str.is_empty() { continue; }
                    let tokens: Vec<&str> = sentence_str.split_whitespace().collect();
                    match parse_sentence(&tokens, pline, cnames) {
                        Ok(op) => ops.push(FlatOp {
                            speaker: speaker.to_string(),
                            op,
                            line: pline,
                        }),
                        Err(_) => {} // flavour text / unrecognised — skip silently
                    }
                }
            }
        }
    };

    while line_no < lines.len() {
        let raw = lines[line_no];
        let l = raw.trim();
        line_no += 1;

        if l.is_empty() { continue; }

        let lower = l.to_lowercase();

        // Act header: "Act I: ..."
        if lower.starts_with("act ") {
            flush_pending(&mut pending_dialogue, &current_speaker, pending_line, &mut ops, &char_names);
            pending_dialogue.clear();
            let tok: Vec<&str> = l.split_whitespace().collect();
            // tok[1] may be "I:" — strip trailing punctuation before parsing
            if let Some(num) = tok.get(1)
                .map(|w| w.trim_end_matches(|c: char| !c.is_alphanumeric()))
                .and_then(|w| parse_roman(w))
            {
                current_act = num;
                current_scene = 1; // reset scene counter
            }
            continue;
        }

        // Scene header: "Scene I: ..."
        if lower.starts_with("scene ") {
            flush_pending(&mut pending_dialogue, &current_speaker, pending_line, &mut ops, &char_names);
            pending_dialogue.clear();
            let tok: Vec<&str> = l.split_whitespace().collect();
            // tok[1] may be "I:" — strip trailing punctuation before parsing
            if let Some(num) = tok.get(1)
                .map(|w| w.trim_end_matches(|c: char| !c.is_alphanumeric()))
                .and_then(|w| parse_roman(w))
            {
                current_scene = num;
                scene_index.entry((current_act, current_scene))
                    .or_insert(ops.len());
            }
            continue;
        }

        // Stage direction: "[...]"
        if l.starts_with('[') && l.ends_with(']') {
            flush_pending(&mut pending_dialogue, &current_speaker, pending_line, &mut ops, &char_names);
            pending_dialogue.clear();
            let inner = &l[1..l.len() - 1];
            match parse_stage_direction(inner, line_no) {
                Ok(op) => ops.push(FlatOp {
                    speaker: String::new(),
                    op,
                    line: line_no,
                }),
                Err(e) => return Err(e),
            }
            continue;
        }

        // Dialogue header: "CharacterName:" (possibly followed by text on same line)
        // Check if line ends with ":"
        if l.ends_with(':') {
            let name_part = l.trim_end_matches(':').trim();
            if let Some(canonical) = as_character_name(name_part) {
                flush_pending(&mut pending_dialogue, &current_speaker, pending_line, &mut ops, &char_names);
                pending_dialogue.clear();
                current_speaker = canonical.to_string();
                continue;
            }
        }

        // Otherwise: dialogue spoken by current_speaker.
        // Accumulate into pending buffer — sentences may span multiple physical lines.
        if current_speaker.is_empty() { continue; }

        if pending_dialogue.is_empty() {
            pending_line = line_no;
        } else {
            pending_dialogue.push(' ');
        }
        pending_dialogue.push_str(l);

        // Extract and parse any complete sentences now present in the buffer.
        flush_pending(&mut pending_dialogue, &current_speaker, pending_line, &mut ops, &char_names);
    }

    // Flush any remaining dialogue at end of file.
    flush_pending(&mut pending_dialogue, &current_speaker, pending_line, &mut ops, &char_names);

    Ok(Play {
        title,
        characters,
        ops,
        scene_index,
    })
}

/// Split a line into sentences at ., !, ? boundaries.
/// Keeps the terminator as the last char of each sentence.
fn split_sentences(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c == '.' || c == '!' || c == '?' {
            let end = i + c.len_utf8();
            let seg = &s[start..end];
            if !seg.trim().is_empty() {
                result.push(seg);
            }
            start = end;
        }
    }
    // Trailing text without terminator
    let tail = &s[start..];
    if !tail.trim().is_empty() {
        result.push(tail);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_assignment() {
        let tokens = vec!["You", "are", "a", "beautiful", "rose"];
        let op = parse_sentence(&tokens, 1, &[]).unwrap();
        match op {
            Op::Assign { expr: Expr::Positive { adjective_count: 1 }, cond: None } => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn parse_nothing() {
        let tokens = vec!["You", "are", "nothing"];
        let op = parse_sentence(&tokens, 1, &[]).unwrap();
        match op {
            Op::Assign { expr: Expr::Nothing, .. } => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn parse_output_char() {
        let tokens = vec!["Open", "your", "mouth"];
        let op = parse_sentence(&tokens, 1, &[]).unwrap();
        match op {
            Op::Output { kind: IoKind::Char, .. } => {}
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test]
    fn parse_goto_scene() {
        let tokens = vec!["Let", "us", "proceed", "to", "scene", "II"];
        let op = parse_sentence(&tokens, 1, &[]).unwrap();
        match op {
            Op::Goto { scene: 2, .. } => {}
            other => panic!("unexpected: {:?}", other),
        }
    }
}
