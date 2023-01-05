use super::{Segment, VMInstruction};
use std::str::FromStr;

impl FromStr for Segment {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "argument" => Ok(Segment::Argument),
            "local" => Ok(Segment::Local),
            "static" => Ok(Segment::Static),
            "constant" => Ok(Segment::Constant),
            "this" => Ok(Segment::This),
            "that" => Ok(Segment::That),
            "pointer" => Ok(Segment::Pointer),
            "temp" => Ok(Segment::Temp),
            _ => Err(s.to_owned()),
        }
    }
}

pub fn parse_instruction(line: &str) -> VMInstruction {
    let instruction_elements = line.trim().split(' ').collect::<Vec<&str>>();

    match instruction_elements[0] {
        "push" => VMInstruction::Push(
            instruction_elements[1].parse().unwrap(),
            instruction_elements[2].parse().unwrap(),
        ),
        "pop" => VMInstruction::Pop(
            instruction_elements[1].parse().unwrap(),
            instruction_elements[2].parse().unwrap(),
        ),
        "add" => VMInstruction::Add,
        "sub" => VMInstruction::Sub,
        "neg" => VMInstruction::Neg,
        "eq" => VMInstruction::Eq,
        "gt" => VMInstruction::Gt,
        "lt" => VMInstruction::Lt,
        "and" => VMInstruction::And,
        "or" => VMInstruction::Or,
        "not" => VMInstruction::Not,
        _ => unimplemented!(),
    }
}
