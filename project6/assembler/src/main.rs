use rustc_hash::FxHashMap;
use std::io::{BufRead, BufReader};

fn main() {
    let mut args = std::env::args();
    args.next();
    let filename = args.next();
    let reader = filename
        .and_then(|x| std::fs::File::open(x).ok())
        .expect("Error reading file. Make sure to provide filename as first argument.");
    let mut lines = BufReader::new(reader);
    lines.fill_buf().unwrap();
    let mut lines = lines.lines().map(|x| x.unwrap());

    // First pass; Strip whitespace and save label locations.
    let mut pc = 0;
    let mut symbol_table: FxHashMap<String, u16> = new_symbol_table();
    let mut file: Vec<String> = Vec::new();
    while let Some(line) = lines.next() {
        let command = strip_whitespace(&line);
        if command.is_empty() {
            continue;
        }
        match &command[..1] {
            "(" => label_command(&command, pc, &mut symbol_table),
            _ => {
                file.push(command.to_owned());
                pc += 1
            }
        }
    }

    // Second pass; Parse A and C instructions
    let mut output = String::with_capacity(file.len() * 17);
    let mut variable_counter = 16;
    for line in file {
        output.push_str(&match &line[..1] {
            "@" => a_command(&line, &mut variable_counter, &mut symbol_table),
            _ => c_command(&line),
        });
        output.push('\n');
    }
    output.pop();

    println!("{}", output);
}

fn strip_whitespace(s: &str) -> &str {
    let comment_start = s.find('/').unwrap_or(s.len());
    let s = &s[..comment_start];
    s.trim()
}

fn label_command(s: &str, pc: usize, symbol_table: &mut FxHashMap<String, u16>) {
    let s = &s[1..s.len() - 1];
    symbol_table.insert(s.to_owned(), pc as u16);
}

fn a_command(
    s: &str,
    variable_counter: &mut u16,
    symbol_table: &mut FxHashMap<String, u16>,
) -> String {
    let s = &s[1..];
    let val = s.parse::<u16>().ok().unwrap_or_else(|| {
        symbol_table.get(s).copied().unwrap_or_else(|| {
            symbol_table.insert(s.to_owned(), *variable_counter);
            *variable_counter += 1;
            symbol_table.get(s).copied().unwrap()
        })
    });

    let mut out = String::with_capacity(16);
    out.push('0');
    out.push_str(&format!("{:015b}", val));
    out
}

fn c_command(s: &str) -> String {
    let mut out = String::with_capacity(16);
    out.push_str("111");
    let (dest_s, rest) = s
        .find("=")
        .map(|i| s.split_at(i))
        .map(|(dest, rest)| (dest, &rest[1..]))
        .unwrap_or(("", &s));
    let (comp_s, jmp_s) = rest
        .find(";")
        .map(|i| rest.split_at(i))
        .map(|(comp, jmp)| (comp, &jmp[1..]))
        .unwrap_or((rest, ""));

    out.push_str(&comp_segment(comp_s));
    out.push_str(&dest_segment(dest_s));
    out.push_str(&jmp_segment(jmp_s));

    out
}

fn comp_segment(s: &str) -> String {
    let mut out = String::with_capacity(7);
    let a_param = s.contains('M');
    out.push(if a_param { '1' } else { '0' });
    // Not the most clever way to do it, but it's fast and quite readable.
    out.push_str(match s {
        "0" => "101010",
        "1" => "111111",
        "-1" => "111010",
        "D" => "001100",
        "A" | "M" => "110000",
        "!D" => "001101",
        "!A" | "!M" => "110001",
        "-D" => "001111",
        "-A" | "-M" => "110011",
        "D+1" => "011111",
        "A+1" | "M+1" => "110111",
        "D-1" => "001110",
        "A-1" | "M-1" => "110010",
        "D+A" | "A+D" | "M+D" | "D+M" => "000010",
        "D-A" | "D-M" => "010011",
        "A-D" | "M-D" => "000111",
        "D&A" | "A&D" | "D&M" | "M&D" => "000000",
        "D|A" | "A|D" | "D|M" | "M|D" => "010101",
        _ => unimplemented!("{}", s),
    });
    out
}

fn dest_segment(s: &str) -> String {
    let mut out: [u8; 3] = *b"000";
    if s.contains('A') {
        out[0] = b'1';
    }
    if s.contains('D') {
        out[1] = b'1';
    }
    if s.contains('M') {
        out[2] = b'1';
    }
    String::from_utf8(Vec::from(out)).unwrap()
}

fn jmp_segment(s: &str) -> String {
    match s {
        "JGT" => "001",
        "JEQ" => "010",
        "JGE" => "011",
        "JLT" => "100",
        "JNE" => "101",
        "JLE" => "110",
        "JMP" => "111",
        _ => "000",
    }
    .to_owned()
}

fn new_symbol_table() -> FxHashMap<String, u16> {
    let mut map = FxHashMap::default();
    map.insert("R0".to_owned(), 0);
    map.insert("R1".to_owned(), 1);
    map.insert("R2".to_owned(), 2);
    map.insert("R3".to_owned(), 3);
    map.insert("R4".to_owned(), 4);
    map.insert("R5".to_owned(), 5);
    map.insert("R6".to_owned(), 6);
    map.insert("R7".to_owned(), 7);
    map.insert("R8".to_owned(), 8);
    map.insert("R9".to_owned(), 9);
    map.insert("R10".to_owned(), 10);
    map.insert("R11".to_owned(), 11);
    map.insert("R12".to_owned(), 12);
    map.insert("R13".to_owned(), 13);
    map.insert("R14".to_owned(), 14);
    map.insert("R15".to_owned(), 15);
    map.insert("SP".to_owned(), 0);
    map.insert("LCL".to_owned(), 1);
    map.insert("ARG".to_owned(), 2);
    map.insert("THIS".to_owned(), 3);
    map.insert("THAT".to_owned(), 4);
    map.insert("SCREEN".to_owned(), 0x4000);
    map.insert("KBD".to_owned(), 0x6000);
    map
}
