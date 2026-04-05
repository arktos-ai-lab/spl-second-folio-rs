use thiserror::Error;

#[derive(Debug, Error)]
pub enum SplError {
    #[error("parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("runtime error: {message}")]
    Runtime { message: String },

    #[error("unknown character: {name}")]
    UnknownCharacter { name: String },

    #[error("character not on stage: {name}")]
    NotOnStage { name: String },

    #[error("character already on stage: {name}")]
    AlreadyOnStage { name: String },

    #[error("stack underflow for character {name}")]
    StackUnderflow { name: String },

    #[error("no opposite found (need exactly 2 characters on stage)")]
    NoOpposite,

    #[error("scene not found: Act {act}, Scene {scene}")]
    SceneNotFound { act: u32, scene: u32 },

    #[error("division by zero")]
    DivisionByZero,

    #[error("invalid output character: code point {code}")]
    InvalidChar { code: i64 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("step limit exceeded after {steps} operations (program may be waiting for input or in an infinite loop)")]
    StepLimitExceeded { steps: u64 },
}

impl SplError {
    pub fn parse(line: usize, msg: impl Into<String>) -> Self {
        SplError::Parse { line, message: msg.into() }
    }
    pub fn runtime(msg: impl Into<String>) -> Self {
        SplError::Runtime { message: msg.into() }
    }
}

pub type Result<T> = std::result::Result<T, SplError>;
