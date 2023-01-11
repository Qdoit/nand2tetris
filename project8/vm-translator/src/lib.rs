#[derive(Debug, Clone)]
pub enum VMInstruction {
    Push(Segment, u16),
    Pop(Segment, u16),
    Function(String, u16),
    Call(String, u16),
    Return,
    Goto(String),
    IfGoto(String),
    Label(String),
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

pub fn compile_file(
    reader: impl io::BufRead,
    writer: &mut impl io::Write,
    filename: &str,
    generate_bootstrap: bool,
) {
    let mut file_data = codegen::FileData::new(filename);
    let mut lines = reader.lines();

    if generate_bootstrap {
        write!(writer, "{}", codegen::init_code(&mut file_data)).unwrap();
    }

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        let line = &line[..line.find('/').unwrap_or(line.len())].trim();
        if line.is_empty() {
            continue;
        }
        let parsed = parser::parse_instruction(line);
        write!(
            writer,
            "{}",
            codegen::codegen_instruction(parsed, &mut file_data,)
        )
        .unwrap();
    }
}

use std::fs;
use std::io;
use std::path;
pub fn parse_args() -> (Vec<path::PathBuf>, Box<dyn io::Write>, bool, bool, bool) {
    let mut args = std::env::args();
    let mut output_file: Option<path::PathBuf> = None;
    let mut input_file_paths: Vec<path::PathBuf> = Vec::new();
    let mut read_from_stdin = false;
    let mut no_bootstrap = false;
    let mut terminate_immiediately = false;
    args.next();
    while let Some(arg) = args.next() {
        let arg2: &str = &arg;
        match arg2 {
            "-o" | "--output" => {
                output_file = args.next().map(path::PathBuf::from);
            }
            "-h" | "--help" => {
                print!("{}", USAGE);
                terminate_immiediately = true;
            }
            "-si" | "--stdin" => {
                read_from_stdin = true;
            }
            "-nb" | "--no-bootstrap" => {
                no_bootstrap = true;
            }
            _ => {
                input_file_paths.push(path::PathBuf::from(arg));
            }
        }
    }

    let output_writer: Box<dyn io::Write> = match output_file {
        Some(x) => Box::new(io::BufWriter::new(fs::File::create(x).unwrap())),
        None => Box::new(io::stdout().lock()),
    };

    (
        input_file_paths,
        output_writer,
        read_from_stdin,
        no_bootstrap,
        terminate_immiediately,
    )
}

pub const USAGE: &str = r#"Usage:
    vm-translator <file> [options]

Options:
  -o <file>, --output <file>            Outputs to [fn] instead of standard output
  -si,       --stdin                    Reads from standard input instead of the given file.
                                        The <file> argument can be omitted.
  -nb,       --no-bootstrap             Stops the translator from emitting bootstrap code
                                        at the beginning of output.
  -h,        --help                     Prints help message
"#;
