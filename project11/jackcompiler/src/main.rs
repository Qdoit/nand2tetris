use std::fs::DirEntry;
use std::path::Path;

use jackc::lexer;
use jackc::parse;

fn main() {
    let mut args = std::env::args();
    args.next();
    let path = match args.next() {
        Some(p) => p,
        _ => {
            println!("incorrect path <3");
            return;
        },
    };

    let path = Path::new(&path);
    if path.is_file() {
        let src = std::fs::read_to_string(path).unwrap();
        let mut lexer = lexer::Lexer::new(&src);
        let out_file = std::fs::File::create(format!(
            "{}.vm",
            path.to_str().unwrap().trim_end_matches(".jack")
        ))
        .unwrap();
        parse::parse(&mut lexer, out_file);
    } else if path.is_dir() {
        for e in path.read_dir().unwrap() {
            let e: DirEntry = e.unwrap();
            if e.path().is_file()
                && e.file_name().into_string().unwrap().rsplit('.').next() == Some("jack")
            {
                let out_file = std::fs::File::create(format!(
                    "{}.vm",
                    e.path().to_str().unwrap().trim_end_matches(".jack")
                ))
                .unwrap();
                let src = std::fs::read_to_string(e.path()).unwrap();
                let mut lexer = lexer::Lexer::new(&src);
                parse::parse(&mut lexer, out_file);
            }
        }
    } else {
        println!("incorrect path <3");
    }
}
