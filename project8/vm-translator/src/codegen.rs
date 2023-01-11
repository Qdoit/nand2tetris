use crate::{Segment, VMInstruction};
pub struct FileData {
    static_index_label_gen: StaticIndexLabelGen,
    file_label_gen: FileLabelGen,
    function_label_gen: FunctionLabelGen,
}

impl FileData {
    pub fn new(filename: &str) -> FileData {
        FileData {
            static_index_label_gen: StaticIndexLabelGen::new(filename.to_owned()),
            file_label_gen: FileLabelGen::new(filename.to_owned()),
            function_label_gen: FunctionLabelGen::new(format!("{filename}.")),
        }
    }
}

pub fn codegen_instruction(i: VMInstruction, file_data: &mut FileData) -> String {
    let FileData {
        static_index_label_gen,
        file_label_gen,
        function_label_gen,
    } = file_data;
    let mut s = format!("// {:?}\n", i);
    s.push_str(&match i {
        VMInstruction::Push(segment, seg_idx) => {
            format!(
                "{}\n\
                     @SP\n\
                     A=M\n\
                     M=D\n\
                     @SP\n\
                     M=M+1\n",
                segment.set_d_to_value_at_index(seg_idx, static_index_label_gen)
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
                segment.set_d_to_target_address(seg_idx, static_index_label_gen)
            )
        }
        VMInstruction::Label(label) => {
            format!("({})\n", function_label_gen.goto_label(&label))
        }
        VMInstruction::Goto(label) => {
            format!("@{}\n0;JMP\n", function_label_gen.goto_label(&label))
        }
        VMInstruction::IfGoto(label) => ifgoto_instruction(&function_label_gen.goto_label(&label)),
        VMInstruction::Function(name, locals_count) => {
            function_instruction(name, locals_count, function_label_gen)
        }
        VMInstruction::Call(name, args_count) => {
            call_instruction(args_count, name, function_label_gen)
        }
        VMInstruction::Return => return_instruction(),
        c @ (VMInstruction::Eq | VMInstruction::Gt | VMInstruction::Lt) => {
            cmp_instruction(c, file_label_gen)
        }
        c @ (VMInstruction::Not | VMInstruction::Neg) => one_arg_instruction(c),
        c => two_arg_arith_logic_instruction(c),
    });
    s
}

fn return_instruction() -> String {
    "@LCL\n\
     D=M\n\
     @5\n\
     D=D-A\n\
     @R13\n\
     M=D\n\
     A=D\n\
     D=M\n\
     @R14\n\
     M=D\n\
     @SP\n\
     AM=M-1\n\
     D=M\n\
     @ARG\n\
     A=M\n\
     M=D\n\
     D=A\n\
     @SP\n\
     M=D+1\n\
     @R13\n\
     AM=M+1\n\
     D=M\n\
     @LCL\n\
     M=D\n\
     @R13\n\
     AM=M+1\n\
     D=M\n\
     @ARG\n\
     M=D\n\
     @R13\n\
     AM=M+1\n\
     D=M\n\
     @THIS\n\
     M=D\n\
     @R13\n\
     AM=M+1\n\
     D=M\n\
     @THAT\n\
     M=D\n\
     @R14\n\
     A=M\n\
     0;JMP\n\
"
    .to_owned()
}

pub fn init_code(file_data: &mut FileData) -> String {
    format!(
        "@256\nD=A\n@SP\nM=D\n{}",
        call_instruction(0, "Sys.init".to_owned(), &mut file_data.function_label_gen)
    )
}

fn call_instruction(args_count: u16, fn_name: String, fn_lg: &mut FunctionLabelGen) -> String {
    let ret_label = fn_lg.next_return();

    let push_d_to_stack = "@SP\n\
                           A=M\n\
                           M=D\n\
                           @SP\n\
                           M=M+1\n";
    let offset = args_count + 5;
    format!(
        "@{ret_label}\n\
             D=A\n\
             {push_d_to_stack}\
             @LCL\n\
             D=M\n\
             {push_d_to_stack}\
             @ARG\n\
             D=M\n\
             {push_d_to_stack}\
             @THIS\n\
             D=M\n\
             {push_d_to_stack}\
             @THAT\n\
             D=M\n\
             {push_d_to_stack}\
             @SP\n\
             D=M\n\
             @LCL\n\
             M=D\n\
             @{offset}\n\
             D=D-A\n\
             @ARG\n\
             M=D\n\
             @{fn_name}\n\
             0;JMP\n\
             ({ret_label})\n"
    )
}

fn function_instruction(
    name: String,
    locals_count: u16,
    function_lg: &mut FunctionLabelGen,
) -> String {
    let mut out = format!("({})\n@SP\nA=M\n", name);
    for _ in 0..locals_count {
        out.push_str(
            "M=0\n\
            @SP\n\
                      AM=M+1\n",
        );
    }

    *function_lg = FunctionLabelGen::new(name);

    out
}

fn ifgoto_instruction(label: &str) -> String {
    format!(
        "@SP\n\
             AM=M-1\n\
             D=M\n\
             @{label}\n\
             D;JNE\n"
    )
}

fn cmp_instruction(op: VMInstruction, label_gen: &mut FileLabelGen) -> String {
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

impl Segment {
    fn set_d_to_value_at_index(&self, idx: u16, static_indexing: &StaticIndexLabelGen) -> String {
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

    fn set_d_to_target_address(&self, idx: u16, static_indexing: &StaticIndexLabelGen) -> String {
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

struct StaticIndexLabelGen {
    filename: String,
}

impl StaticIndexLabelGen {
    fn new(filename: String) -> StaticIndexLabelGen {
        StaticIndexLabelGen { filename }
    }

    fn nth(&self, i: usize) -> String {
        format!("{}.{}", self.filename, i)
    }
}

struct FileLabelGen {
    filename: String,
    comparison_count: usize,
}

impl FileLabelGen {
    fn new(filename: String) -> FileLabelGen {
        FileLabelGen {
            filename,
            comparison_count: 0,
        }
    }

    fn next_cmp_label(&mut self) -> String {
        let out = format!("CMP{}{}", self.filename.clone(), self.comparison_count);
        self.comparison_count += 1;
        out
    }
}

struct FunctionLabelGen {
    function_name: String,
    return_count: usize,
}

impl FunctionLabelGen {
    fn new(function_name: String) -> FunctionLabelGen {
        FunctionLabelGen {
            function_name,
            return_count: 0,
        }
    }

    fn goto_label(&self, label: &str) -> String {
        format!("{}${label}", self.function_name)
    }

    fn next_return(&mut self) -> String {
        let out = format!("{}$ret.{}", self.function_name, self.return_count);
        self.return_count += 1;
        out
    }
}
