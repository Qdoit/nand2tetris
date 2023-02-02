use std::fs::DirEntry;
use std::path::Path;

use jackanalyser::lexer;
use jackanalyser::parse;

fn main() {
    let mut args = std::env::args();
    args.next();
    let path = args.next().unwrap();
    let path = Path::new(&path);
    if path.is_file() {
        let src = std::fs::read_to_string(path).unwrap();
        let mut lexer = lexer::Lexer::new(&src);
        parse::parse(&mut lexer);  
    } else if path.is_dir() {
        for e in path.read_dir().unwrap() {
            let e: DirEntry = e.unwrap();
            if e.path().is_file() && e.file_name().into_string().unwrap().rsplit('.').next() == Some("jack") {
                let src = std::fs::read_to_string(e.path()).unwrap();
                let mut lexer = lexer::Lexer::new(&src);
                parse::parse(&mut lexer);  
            }
        }
    } else {
        println!("incorrect path <3");
    }

}
