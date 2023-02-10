use std::{collections::HashMap, io::Write};

use crate::lexer;

pub struct CodeGen<'a, O: Write> {
    out: O,
    pub class_name: Vec<u8>,
    pub class_table: HashMap<&'a [u8], SymbolEntry<'a>>,
    pub subroutine_table: HashMap<&'a [u8], SymbolEntry<'a>>,
    label_idx: usize,
    counters: [u16; 4],
}

impl<'a, O: Write> CodeGen<'a, O> {
    pub fn new(out: O) -> Self {
        CodeGen {
            out,
            class_name: Vec::new(),
            class_table: HashMap::new(),
            subroutine_table: HashMap::new(),
            label_idx: 0,
            counters: [0; 4],
        }
    }

    pub fn reset_class(&mut self) {
        self.class_table.clear();
        self.class_name.clear();
        self.counters[Kind::Field as usize] = 0;
        self.counters[Kind::Static as usize] = 0;
    }

    pub fn reset_subroutine(&mut self) {
        self.subroutine_table.clear();
        self.counters[Kind::Argument as usize] = 0;
        self.counters[Kind::Local as usize] = 0;
    }

    pub fn push(&mut self, segment: Segment, idx: u16) {
        writeln!(self.out, "push {} {}", u8stostr(segment.as_str()), idx).unwrap();
    }

    pub fn pop(&mut self, segment: Segment, idx: u16) {
        writeln!(self.out, "pop {} {}", u8stostr(segment.as_str()), idx).unwrap();
    }

    pub fn arithmetic(&mut self, instruction: ArithmeticInstruction) {
        writeln!(self.out, "{}", u8stostr(instruction.as_str())).unwrap();
    }

    pub fn next_label_idx(&mut self) -> usize {
        self.label_idx += 1;
        self.label_idx
    }

    pub fn label(&mut self, label: &[u8]) {
        writeln!(self.out, "label {}", u8stostr(label)).unwrap();
    }

    pub fn goto(&mut self, label: &[u8]) {
        writeln!(self.out, "goto {}", u8stostr(label)).unwrap();
    }

    pub fn if_goto(&mut self, label: &[u8]) {
        writeln!(self.out, "if-goto {}", u8stostr(label)).unwrap();
    }

    pub fn call(&mut self, name: &[u8], args_count: u16) {
        writeln!(self.out, "call {} {args_count}", u8stostr(name)).unwrap();
    }

    pub fn function(&mut self, name: &[u8], args_count: u16) {
        writeln!(self.out, "function {} {args_count}", u8stostr(name)).unwrap();
    }

    pub fn write_return(&mut self) {
        writeln!(self.out, "return").unwrap();
    }

    pub fn get_symbol(&self, name: &[u8]) -> Option<&SymbolEntry<'a>> {
        self.subroutine_table
            .get(name)
            .or_else(|| self.class_table.get(name))
    }

    pub fn get_symbol_mut(&mut self, name: &[u8]) -> Option<&mut SymbolEntry<'a>> {
        self.subroutine_table
            .get_mut(name)
            .or_else(|| self.class_table.get_mut(name))
    }

    pub fn add_symbol(&mut self, name: &'a [u8], ty: Ty<'a>, kind: Kind) {
        let idx = self.counters[kind as usize];
        self.counters[kind as usize] += 1;

        match kind {
            Kind::Field | Kind::Static => {
                self.class_table.insert(name, SymbolEntry { ty, kind, idx })
            }
            Kind::Local | Kind::Argument => self
                .subroutine_table
                .insert(name, SymbolEntry { ty, kind, idx }),
        };
    }

    pub fn fields_count(&self) -> u16 {
        self.counters[Kind::Field as usize]
    }

    pub fn class_name(&self) -> &[u8] {
        &self.class_name
    }
}

fn u8stostr(s: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(s) }
}

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticInstruction {
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

impl ArithmeticInstruction {
    fn as_str(&self) -> &'static [u8] {
        use ArithmeticInstruction::*;
        match self {
            Add => b"add",
            Sub => b"sub",
            Neg => b"neg",
            Eq => b"eq",
            Gt => b"gt",
            Lt => b"lt",
            And => b"and",
            Or => b"or",
            Not => b"not",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Segment {
    Constant,
    Argument,
    Local,
    Static,
    This,
    That,
    Pointer,
    Temp,
}

impl From<Kind> for Segment {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Field => Segment::This,
            Kind::Local => Segment::Local,
            Kind::Static => Segment::Static,
            Kind::Argument => Segment::Argument,
        }
    }
}

impl Segment {
    fn as_str(&self) -> &'static [u8] {
        use Segment::*;
        match self {
            Constant => b"constant",
            Argument => b"argument",
            Local => b"local",
            Static => b"static",
            This => b"this",
            That => b"that",
            Pointer => b"pointer",
            Temp => b"temp",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolEntry<'a> {
    pub ty: Ty<'a>,
    pub kind: Kind,
    pub idx: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum Ty<'a> {
    Int,
    Bool,
    Char,
    Class(&'a [u8]),
}

impl<'a> Ty<'a> {
    pub fn from_token(tok: lexer::Token<'a>) -> Self {
        use lexer::{Keyword, Token};
        match tok {
            Token::Ident(s) => Self::Class(s),
            Token::Keyword(Keyword::Int) => Self::Int,
            Token::Keyword(Keyword::Char) => Self::Char,
            Token::Keyword(Keyword::Boolean) => Self::Bool,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Field = 0,
    Static,
    Argument,
    Local,
}

impl TryFrom<lexer::Keyword> for Kind {
    fn try_from(value: lexer::Keyword) -> Result<Self, ()> {
        use lexer::Keyword::*;
        match value {
            Field => Ok(Kind::Field),
            Static => Ok(Kind::Static),
            Var => Ok(Kind::Local),
            _ => Err(()),
        }
    }

    type Error = ();
}
