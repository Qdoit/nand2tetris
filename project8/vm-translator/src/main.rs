use std::{
    fs,
    io::{self, BufReader},
    process::ExitCode,
};
use vm_translator::{compile_file, parse_args, USAGE};

fn main() -> ExitCode {
    let (
        input_file_paths,
        mut output_writer,
        read_from_stdin,
        no_bootstrap,
        terminate_immiediately,
    ) = parse_args();

    if terminate_immiediately {
        return ExitCode::SUCCESS;
    }

    let mut is_first = !no_bootstrap;

    for file_path in input_file_paths.iter() {
        let filename = file_path.file_name().unwrap().to_str().unwrap();
        let filename = filename.split('.').next().unwrap();

        let mut input_reader = BufReader::new(match fs::File::open(file_path) {
            Err(_) => {
                println!("Couldn't open file: {}", file_path.display());
                return ExitCode::FAILURE;
            }
            Ok(x) => x,
        });

        compile_file(&mut input_reader, &mut output_writer, filename, is_first);
        is_first = false;
    }

    if input_file_paths.is_empty() {
        if read_from_stdin {
            compile_file(io::stdin().lock(), &mut output_writer, "noname", is_first);
        } else {
            print!("{}", USAGE);
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
