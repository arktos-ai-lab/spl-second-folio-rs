/// SPL interpreter — state machine over a flattened operation list.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::ast::*;
use crate::error::{Result, SplError};

// ---------------------------------------------------------------------------
// Per-character state
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct Character {
    pub value: i64,
    pub stack: Vec<i64>,
}

impl Character {
    pub fn push(&mut self, v: i64) {
        self.stack.push(v);
    }
    pub fn pop(&mut self, name: &str) -> Result<()> {
        self.value = self.stack.pop().ok_or_else(|| SplError::StackUnderflow {
            name: name.to_string(),
        })?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Interpreter state
// ---------------------------------------------------------------------------

pub struct State {
    /// All declared characters (lowercase name → state)
    pub characters: HashMap<String, Character>,
    /// Names of characters currently on stage (lowercase, ordered)
    pub on_stage: Vec<String>,
    /// The result of the last Question sentence
    pub global_boolean: bool,
    /// Current position in the flattened ops list
    pub pc: usize,
}

impl State {
    pub fn new(play: &Play) -> Self {
        let mut characters = HashMap::new();
        for decl in &play.characters {
            characters.insert(decl.name.clone(), Character::default());
        }
        State {
            characters,
            on_stage: Vec::new(),
            global_boolean: false,
            pc: 0,
        }
    }

    /// The opposite of `speaker` — valid only when exactly 2 on stage.
    pub fn opposite(&self, speaker: &str) -> Result<&str> {
        if self.on_stage.len() != 2 {
            return Err(SplError::NoOpposite);
        }
        let other = self.on_stage.iter().find(|n| n.as_str() != speaker);
        other.map(|s| s.as_str()).ok_or(SplError::NoOpposite)
    }

    pub fn character_mut(&mut self, name: &str) -> Result<&mut Character> {
        self.characters.get_mut(name).ok_or_else(|| SplError::UnknownCharacter {
            name: name.to_string(),
        })
    }

    pub fn character(&self, name: &str) -> Result<&Character> {
        self.characters.get(name).ok_or_else(|| SplError::UnknownCharacter {
            name: name.to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Expression evaluation
// ---------------------------------------------------------------------------

fn eval_expr(expr: &Expr, state: &State, speaker: &str) -> Result<i64> {
    match expr {
        Expr::FirstPerson => Ok(state.character(speaker)?.value),
        Expr::SecondPerson => {
            let opp = state.opposite(speaker)?;
            Ok(state.character(opp)?.value)
        }
        Expr::CharRef(name) => Ok(state.character(name)?.value),
        Expr::Nothing => Ok(0),
        Expr::Positive { adjective_count } => Ok(2_i64.wrapping_pow(*adjective_count)),
        Expr::Negative { adjective_count } => Ok(2_i64.wrapping_pow(*adjective_count).wrapping_neg()),
        Expr::Unary { op, operand } => {
            let v = eval_expr(operand, state, speaker)?;
            Ok(match op {
                UnaryOp::Square     => v.wrapping_mul(v),
                UnaryOp::Cube       => v.wrapping_mul(v).wrapping_mul(v),
                UnaryOp::SquareRoot => (v as f64).sqrt() as i64,
                UnaryOp::Factorial  => factorial(v)?,
                UnaryOp::Twice      => v.wrapping_mul(2),
            })
        }
        Expr::Binary { op, left, right } => {
            let l = eval_expr(left, state, speaker)?;
            let r = eval_expr(right, state, speaker)?;
            Ok(match op {
                BinaryOp::Sum        => l.wrapping_add(r),
                BinaryOp::Difference => l.wrapping_sub(r),
                BinaryOp::Product    => l.wrapping_mul(r),
                BinaryOp::Quotient   => {
                    if r == 0 { return Err(SplError::DivisionByZero); }
                    l / r  // truncated toward zero, matching Python's int()
                }
                BinaryOp::Remainder  => {
                    if r == 0 { return Err(SplError::DivisionByZero); }
                    l % r
                }
            })
        }
    }
}

fn factorial(n: i64) -> Result<i64> {
    if n < 0 {
        return Err(SplError::runtime("factorial of negative number"));
    }
    let mut acc = 1i64;
    for i in 2..=n {
        acc = acc.saturating_mul(i);
    }
    Ok(acc)
}

fn eval_cmp(cmp: &Comparison, state: &State, speaker: &str) -> Result<bool> {
    let l = eval_expr(&cmp.left, state, speaker)?;
    let r = eval_expr(&cmp.right, state, speaker)?;
    Ok(match cmp.op {
        CmpOp::Equal          => l == r,
        CmpOp::NotEqual       => l != r,
        CmpOp::GreaterThan    => l > r,
        CmpOp::LessThan       => l < r,
        CmpOp::GreaterOrEqual => l >= r,
        CmpOp::LessOrEqual    => l <= r,
    })
}

// ---------------------------------------------------------------------------
// Main interpreter
// ---------------------------------------------------------------------------

pub struct Interpreter<R: BufRead, W: Write> {
    pub play: Play,
    pub state: State,
    pub input: R,
    pub output: W,
    /// If true, trace each operation to stderr
    pub trace: bool,
    /// When set, trace lines are also appended here (for the web UI)
    pub trace_buf: Vec<String>,
    /// Maximum number of operations to execute (0 = unlimited)
    pub max_steps: u64,
    /// When true, reading past EOF returns -1 (matching original spl2c/C behaviour).
    /// When false, reading past EOF returns WaitingForInput (interactive/web mode).
    pub eof_as_minus_one: bool,
}

impl<R: BufRead, W: Write> Interpreter<R, W> {
    pub fn new(play: Play, input: R, output: W) -> Self {
        let state = State::new(&play);
        Interpreter { play, state, input, output, trace: false, trace_buf: Vec::new(), max_steps: 0, eof_as_minus_one: false }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut steps: u64 = 0;
        while self.state.pc < self.play.ops.len() {
            if self.max_steps > 0 {
                steps += 1;
                if steps > self.max_steps {
                    return Err(crate::error::SplError::StepLimitExceeded { steps });
                }
            }

            let idx = self.state.pc;
            self.state.pc += 1;
            let flat = self.play.ops[idx].clone();

            if self.trace {
                let line = format!("[line {:4}]  op {:4}  spk={:<15}  {:?}",
                    flat.line, idx,
                    if flat.speaker.is_empty() { "<stage>" } else { &flat.speaker },
                    flat.op);
                eprintln!("{}", line);
                self.trace_buf.push(line);
            }

            self.execute(&flat)?;
        }
        Ok(())
    }

    fn execute(&mut self, flat: &FlatOp) -> Result<()> {
        let speaker = flat.speaker.clone();
        match &flat.op {
            Op::Enter(names) => {
                for name in names {
                    // If already on stage, silently skip (matches zmbc behaviour;
                    // common when a scene that opens with [Enter] is jumped to as a loop head).
                    if self.state.on_stage.contains(name) {
                        continue;
                    }
                    // Auto-declare undeclared characters (permissive mode)
                    self.state.characters.entry(name.clone()).or_default();
                    self.state.on_stage.push(name.clone());
                }
            }
            Op::Exit(name) => {
                if !self.state.on_stage.contains(name) {
                    return Err(SplError::NotOnStage { name: name.clone() });
                }
                self.state.on_stage.retain(|n| n != name);
            }
            Op::Exeunt(names) => {
                if names.is_empty() {
                    self.state.on_stage.clear();
                } else {
                    for name in names {
                        self.state.on_stage.retain(|n| n != name);
                    }
                }
            }
            Op::Assign { expr, cond } => {
                if !self.check_cond(*cond) { return Ok(()); }
                let val = eval_expr(expr, &self.state, &speaker)?;
                let opp = self.state.opposite(&speaker)?.to_string();
                self.state.character_mut(&opp)?.value = val;
            }
            Op::Output { kind, cond } => {
                if !self.check_cond(*cond) { return Ok(()); }
                let opp = self.state.opposite(&speaker)?.to_string();
                let val = self.state.character(&opp)?.value;
                match kind {
                    IoKind::Numeric => write!(self.output, "{}", val)?,
                    IoKind::Char => {
                        let code = u32::try_from(val).ok()
                            .and_then(char::from_u32)
                            .ok_or(SplError::InvalidChar { code: val })?;
                        write!(self.output, "{}", code)?;
                    }
                }
                self.output.flush()?;
            }
            Op::Input { kind, cond } => {
                if !self.check_cond(*cond) { return Ok(()); }
                let opp = self.state.opposite(&speaker)?.to_string();
                let val = match kind {
                    IoKind::Numeric => self.read_number()?,
                    IoKind::Char    => self.read_char()?,
                };
                self.state.character_mut(&opp)?.value = val;
            }
            Op::Push { expr, cond } => {
                if !self.check_cond(*cond) { return Ok(()); }
                let val = eval_expr(expr, &self.state, &speaker)?;
                let opp = self.state.opposite(&speaker)?.to_string();
                self.state.character_mut(&opp)?.push(val);
            }
            Op::Pop { cond } => {
                if !self.check_cond(*cond) { return Ok(()); }
                let opp = self.state.opposite(&speaker)?.to_string();
                let name = opp.clone();
                self.state.character_mut(&opp)?.pop(&name)?;
            }
            Op::Question { cmp } => {
                let result = eval_cmp(cmp, &self.state, &speaker)?;
                self.state.global_boolean = result;
            }
            Op::Goto { act, scene, cond } => {
                if !self.check_cond(*cond) { return Ok(()); }
                // Scene-only gotos use current act
                let act_num = if *act == 1 { self.current_act() } else { *act };
                let key = (act_num, *scene);
                match self.play.scene_index.get(&key) {
                    Some(&target) => self.state.pc = target,
                    None => return Err(SplError::SceneNotFound { act: act_num, scene: *scene }),
                }
            }
            // SPL-F2 ornamental sentence — explicit no-op, no effect on machine state.
            Op::Ornament => {}
        }
        Ok(())
    }

    fn check_cond(&self, cond: Option<Cond>) -> bool {
        match cond {
            None => true,
            Some(Cond::IfSo)  => self.state.global_boolean,
            Some(Cond::IfNot) => !self.state.global_boolean,
        }
    }

    fn current_act(&self) -> u32 {
        // Find which act contains the current pc
        // Heuristic: scan scene_index for the largest act <= pc
        let mut best_act = 1u32;
        for (&(act, _scene), &idx) in &self.play.scene_index {
            if idx <= self.state.pc { best_act = best_act.max(act); }
        }
        best_act
    }

    fn read_number(&mut self) -> Result<i64> {
        let mut buf = String::new();
        self.input.read_line(&mut buf)?;
        buf.trim().parse::<i64>().map_err(|_| {
            SplError::runtime(format!("expected integer, got {:?}", buf.trim()))
        })
    }

    fn read_char(&mut self) -> Result<i64> {
        let mut buf = [0u8; 1];
        use std::io::Read;
        match self.input.read(&mut buf)? {
            0 => {
                if self.eof_as_minus_one {
                    Ok(-1)
                } else {
                    Err(crate::error::SplError::WaitingForInput)
                }
            }
            _ => Ok(buf[0] as i64),
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience: run from string
// ---------------------------------------------------------------------------

pub fn run_play(source: &str) -> Result<String> {
    use crate::parser::parse_play;
    let play = parse_play(source)?;
    let input  = io::Cursor::new(Vec::<u8>::new());
    let output = Vec::<u8>::new();
    let mut interp = Interpreter::new(play, io::BufReader::new(input), output);
    interp.run()?;
    Ok(String::from_utf8_lossy(&interp.output).into_owned())
}

pub fn run_play_with_input(source: &str, stdin: &str) -> Result<String> {
    use crate::parser::parse_play;
    let play = parse_play(source)?;
    let input  = io::Cursor::new(stdin.as_bytes().to_vec());
    let output = Vec::<u8>::new();
    let mut interp = Interpreter::new(play, io::BufReader::new(input), output);
    interp.run()?;
    Ok(String::from_utf8_lossy(&interp.output).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factorial_values() {
        assert_eq!(factorial(0).unwrap(), 1);
        assert_eq!(factorial(5).unwrap(), 120);
        assert_eq!(factorial(10).unwrap(), 3628800);
    }

    #[test]
    fn expr_noun_phrase() {
        // No-adjective noun → 2^0 = 1
        let expr = Expr::positive(0);
        // Can't test without State easily here; value encoding is tested indirectly
        match expr {
            Expr::Positive { adjective_count: 0 } => {}
            _ => panic!(),
        }
    }
}
