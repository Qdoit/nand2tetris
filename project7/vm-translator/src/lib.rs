#[derive(Debug, Clone)]
pub enum VMInstruction {
    Push(Segment, u16),
    Pop(Segment, u16),
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum Segment {
    Argument = 0,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

pub mod codegen;
pub mod parser;
