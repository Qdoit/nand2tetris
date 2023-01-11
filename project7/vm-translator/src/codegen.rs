use crate::{Segment, VMInstruction};

pub struct StaticIndexing {
    pub filename: String,
    pub count: usize,
}

impl StaticIndexing {
    pub fn new(filename: String) -> StaticIndexing {
        StaticIndexing { filename, count: 0 }
    }

    pub fn nth(&self, i: usize) -> String {
        format!("{}.{}", self.filename, i)
    }

    pub fn next_id(&mut self) -> String {
        let out = self.nth(self.count);
        self.count += 1;
        out
    }
}

pub struct LabelGenerator {
    pub filename: String,
    pub comparison_count: usize,
}

impl LabelGenerator {
    pub fn new(filename: String) -> LabelGenerator {
        LabelGenerator {
            filename,
            comparison_count: 0,
        }
    }

    pub fn next_cmp_label(&mut self) -> String {
        let out = format!("CMP{}{}", self.filename.clone(), self.comparison_count);
        self.comparison_count += 1;
        out
    }
}

impl Segment {
    fn set_d_to_val(&self, idx: u16, static_indexing: &StaticIndexing) -> String {
        match self {
            Segment::Constant => {
                format!(
                    "@{}\n\
                         D=A",
                    idx
                )
            }
            Segment::Local => indirect_address_set("LCL", idx),
            Segment::Argument => indirect_address_set("ARG", idx),
            Segment::This => indirect_address_set("THIS", idx),
            Segment::That => indirect_address_set("THAT", idx),
            Segment::Temp => {
                let idx = idx + 5;
                format!(
                    "@{}\n\
                         D=M",
                    idx
                )
            }
            Segment::Pointer => {
                let idx = idx + 3;
                format!(
                    "@{}\n\
                         D=M",
                    idx
                )
            }
            Segment::Static => {
                let variable_name = static_indexing.nth(idx as usize);
                format!(
                    "@{}\n\
                         D=M",
                    variable_name
                )
            }
        }
    }

    fn set_d_to_target_addr(&self, idx: u16, static_indexing: &StaticIndexing) -> String {
        match self {
            Segment::Constant => unimplemented!(),
            Segment::Local => indirect_address_get("LCL", idx),
            Segment::Argument => indirect_address_get("ARG", idx),
            Segment::This => indirect_address_get("THIS", idx),
            Segment::That => indirect_address_get("THAT", idx),
            Segment::Temp => {
                let idx = idx + 5;
                format!("@{}\nD=A", idx)
            }
            Segment::Pointer => {
                let idx = idx + 3;
                format!("@{}\nD=A", idx)
            }
            Segment::Static => {
                let variable_name = static_indexing.nth(idx as usize);
                format!("@{}\nD=A", variable_name)
            }
        }
    }
}

fn indirect_address_get(base: &str, offset: u16) -> String {
    format!(
        "@{}\n\
            D=A\n\
            @{}\n\
            D=M+D",
        offset, base
    )
}

fn indirect_address_set(base: &str, offset: u16) -> String {
    format!(
        "@{}\n\
            D=A\n\
            @{}\n\
            A=M+D\n\
            D=M",
        offset, base
    )
}

pub fn codegen_instruction(
    i: VMInstruction,
    indexing: &mut StaticIndexing,
    labels_generator: &mut LabelGenerator,
) -> String {
    let mut s = format!("// {:?}\n", i);
    s.push_str(
    &match i {
        VMInstruction::Push(segment, seg_idx) => {
            format!(
                "{}\n\
                     @SP\n\
                     A=M\n\
                     M=D\n\
                     @SP\n\
                     M=M+1\n",
                segment.set_d_to_val(seg_idx, &indexing)
            )
        }
        VMInstruction::Pop(segment, seg_idx) => {
            format!(
                "{}\n\
                     @R13\n\
                     M=D\n\
                     @SP\n\
                     AM=M-1\n\
                     D=M\n\
                     @R13\n\
                     A=M\n\
                     M=D\n",
                segment.set_d_to_target_addr(seg_idx, &indexing)
            )
        }
        c @ (VMInstruction::Eq | VMInstruction::Gt | VMInstruction::Lt) => {
            cmp_instruction(c, labels_generator)
        }
        c @ (VMInstruction::Not | VMInstruction::Neg) => one_arg_instruction(c),
        c => two_arg_arith_logic_instruction(c),
    });
    s
}

fn cmp_instruction(op: VMInstruction, label_gen: &mut LabelGenerator) -> String {
    let label = label_gen.next_cmp_label();
    let operation = match op {
        VMInstruction::Eq => "D;JEQ",
        VMInstruction::Lt => "D;JLT",
        VMInstruction::Gt => "D;JGT",
        _ => unreachable!(),
    };
    format!(
        "@SP\n\
             AM=M-1\n\
             D=M\n\
             @SP\n\
             AM=M-1\n\
             D=M-D\n\
             @SP
             A=M
             M=-1
             @{label}\n\
             {operation}\n\
             @SP\n\
             A=M\n\
             M=0\n\
             ({label})\n\
             @SP\n\
             M=M+1\n"
    )
}

fn two_arg_arith_logic_instruction(op: VMInstruction) -> String {
    let operation = match op {
        VMInstruction::Add => "M=D+M",
        VMInstruction::Sub => "M=M-D",
        VMInstruction::And => "M=M&D",
        VMInstruction::Or => "M=M|D",
        _ => unreachable!(),
    };
    format!(
        "@SP\n\
             A=M-1\n\
             D=M\n\
             A=A-1\n\
             {operation}\n\
             @SP\n\
             M=M-1\n"
    )
}

fn one_arg_instruction(op: VMInstruction) -> String {
    let operation = match op {
        VMInstruction::Not => "M=!M",
        VMInstruction::Neg => "M=-M",
        _ => unreachable!(),
    };
    format!(
        "@SP\n\
             A=M-1\n\
             {operation}\n"
    )
}
