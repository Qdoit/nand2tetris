use codegen::codegen_instruction;
use parser::parse_instruction;
use std::fs::File;
use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    process::ExitCode,
};
use vm_translator::{
    codegen::{LabelGenerator, StaticIndexing},
    *,
};

fn main() -> ExitCode {
    let mut args = std::env::args();
    let mut output_file: Option<String> = None;
    let mut input_filename: Option<String> = None;
    let mut read_from_std = false;
    args.next();
    while let Some(arg) = args.next() {
        let arg2: &str = &arg;
        match arg2 {
            "-o" | "--output" => {
                output_file = args.next();
            }
            "-h" | "--help" => {
                print!("{}", USAGE);
                return ExitCode::SUCCESS;
            }
            "-si" | "--stdin" => {
                read_from_std = true;
            }
            _ => {
                input_filename = Some(arg);
            }
        }
    }

    let input_reader: Box<dyn BufRead> = if read_from_std {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::new(match input_filename {
            Some(ref x) => match File::open(x) {
                Err(_) => {
                    println!("Couldn't open file: {}", x);
                    return ExitCode::FAILURE;
                }
                Ok(x) => x,
            },
            None => {
                print!("{}", USAGE);
                return ExitCode::FAILURE;
            }
        }))
    };

    let mut output_writer: Box<dyn Write> = match output_file {
        Some(x) => Box::new(BufWriter::new(File::create(x).unwrap())),
        None => Box::new(io::stdout().lock()),
    };
    
    let alternative_string = "noname.vm".to_owned();
    let input_file_name = std::path::Path::new(input_filename.as_ref().unwrap_or(&alternative_string))
        .file_name()
        .unwrap();
    compile_file(
        input_reader,
        &mut output_writer,
        input_file_name.to_str().unwrap(),
    );

    ExitCode::SUCCESS
}

fn compile_file(reader: Box<dyn BufRead>, writer: &mut dyn Write, filename: &str) {
    let mut indexing = StaticIndexing::new(filename.to_owned());
    let mut labels_generator = LabelGenerator::new(filename.to_owned());
    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        let line = &line[..line.find('/').unwrap_or(line.len())].trim();
        if line.len() == 0 {
            continue;
        }
        let parsed = dbg!(parse_instruction(&line));
        write!(
            writer,
            "{}",
            codegen_instruction(parsed, &mut indexing, &mut labels_generator)
        )
        .unwrap();
    }
}

const USAGE: &str = r#"Usage:
    vm-translator <file> [options]

Options:
  -o <file>, --output <file>            Outputs to [fn] instead of standard output
  -si,       --stdin                    Read from standard input instead of the given file.
                                        The <file> argument can be omitted.
  -h,        --help                     Prints help message
"#;
