#[derive(Clone)]
pub struct Lexer<'a> {
    text: &'a [u8],
    idx: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Lexer<'a> {
        Lexer {
            text: text.as_bytes(),
            idx: 0,
        }
    }

    pub fn next(&mut self) -> Option<Token<'a>> {
        if self.idx == self.text.len() {
            return None;
        }
        self.seek_while(|c| c.is_ascii_whitespace());
        while self.text.get(self.idx) == Some(&b'/') && self.text.get(self.idx + 1) == Some(&b'/') {
            self.seek_while(|c| c != b'\n');
            self.idx += 1;

            self.seek_while(|c| c.is_ascii_whitespace());
        }
        
        while self.text.get(self.idx) == Some(&b'/') && self.text.get(self.idx + 1) == Some(&b'*') {
            while !(self.text.get(self.idx) == Some(&b'*') && self.text.get(self.idx + 1) == Some(&b'/')) {
                self.idx += 1;
            }
            self.idx += 2;
            self.seek_while(|c| c.is_ascii_whitespace());
        }


        match self.text[self.idx] {
            b'"' => {
                self.idx += 1;
                let r = Some(Token::StringConstant(self.seek_while(|c| c != b'"')));
                self.idx += 1;
                r
            },
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => 
                Some(Token::IntConstant(
                    unsafe {
                        std::str::from_utf8_unchecked(self.seek_while(|c| c.is_ascii_digit()))
                    }
                    .parse::<u16>()
                    .ok()?,
                )),
            x if is_symbol(x) => {
                self.idx += 1;
                Some(Token::Symbol(x))
            }
            _ => {
                let s = self.seek_while(|c| !c.is_ascii_whitespace() && !is_symbol(c));
                if s == b"" {
                    None
                } else {
                Some(Keyword::new(s).map(Token::Keyword).unwrap_or_else(|| Token::Ident(s)))
                }
            }
        }
    }

    pub fn peek(&self) -> Option<Token<'a>> {
        if self.idx == self.text.len() {
            return None;
        }
        let mut idx = self.peek_while(self.idx, |c| c.is_ascii_whitespace()).1;
        while self.text.get(idx) == Some(&b'/') && self.text.get(idx + 1) == Some(&b'/') {
            idx = self.peek_while(idx, |c| c != b'\n').1;
            idx += 1;

            idx = self.peek_while(idx, |c| c.is_ascii_whitespace()).1;
        }
        
        while self.text.get(idx) == Some(&b'/') && self.text.get(idx + 1) == Some(&b'*') {
            while !(self.text.get(idx) == Some(&b'*') && self.text.get(idx + 1) == Some(&b'/')) {
                idx += 1;
            }
            idx += 2;
            idx = self.peek_while(idx, |c| c.is_ascii_whitespace()).1;
        }

        match self.text[idx] {
            b'"' => {
                let r = Some(Token::StringConstant(self.peek_while(idx + 1, |c| c != b'"').0));
                r
            },
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => 
                Some(Token::IntConstant(
                    unsafe {
                        std::str::from_utf8_unchecked(self.peek_while(idx, |c| c.is_ascii_digit()).0)
                    }
                    .parse::<u16>()
                    .ok()?,
                )),
            x if is_symbol(x) => {
                Some(Token::Symbol(x))
            }
            _ => {
                let s = self.peek_while(idx, |c| !c.is_ascii_whitespace() && !is_symbol(c)).0;
                Some(Keyword::new(s).map(Token::Keyword).unwrap_or_else(|| Token::Ident(s)))
            }
        }
    }

    fn peek_while(&self, start_idx: usize, mut predicate: impl FnMut(u8) -> bool) -> (&'a [u8], usize) {
        let text = self.text;
        let mut end_idx = start_idx;

        while predicate(text[end_idx]) {
            end_idx += 1;
            if end_idx == text.len() {
                break;
            }
        }

        (&text[start_idx..end_idx], end_idx)
    }

   fn seek_while(&mut self, mut predicate: impl FnMut(u8) -> bool) -> &'a [u8] {
        let text = self.text;
        let start_idx = self.idx;

        while predicate(text[self.idx]) {
            self.idx += 1;
            if self.idx == text.len() {
                break;
            }
        }

        &text[start_idx..self.idx]
    }
}

#[derive(PartialEq, Eq)]
pub enum Token<'a> {
    IntConstant(u16),
    StringConstant(&'a [u8]),
    Ident(&'a [u8]),
    Keyword(Keyword),
    Symbol(u8),
}



impl<'a> std::fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntConstant(arg0) => f.debug_tuple("IntConstant").field(arg0).finish(),
            Self::StringConstant(arg0) => write!(f, "StringConstant({:?})", std::str::from_utf8(arg0)),
            Self::Ident(arg0) => write!(f, "Ident({:?})", std::str::from_utf8(arg0)),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Symbol(arg0) => write!(f, "Symbol('{}')", *arg0 as char),
        }
    }
}

fn is_symbol(c: u8) -> bool {
    match c {
        b'{' | b'}' | b'(' | b')' | b'[' | b']' | b'.' | b',' | b';' | b'+' | b'-' | b'*'
        | b'/' | b'&' | b'|' | b'<' | b'>' | b'=' | b'~' => true,
        _ => false,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}

impl Keyword {
    fn new(s: &[u8]) -> Option<Keyword> {
        match s {
            b"class" => Some(Keyword::Class),
            b"constructor" => Some(Keyword::Constructor),
            b"function" => Some(Keyword::Function),
            b"method" => Some(Keyword::Method),
            b"field" => Some(Keyword::Field),
            b"static" => Some(Keyword::Static),
            b"var" => Some(Keyword::Var),
            b"int" => Some(Keyword::Int),
            b"char" => Some(Keyword::Char),
            b"boolean" => Some(Keyword::Boolean),
            b"void" => Some(Keyword::Void),
            b"true" => Some(Keyword::True),
            b"false" => Some(Keyword::False),
            b"null" => Some(Keyword::Null),
            b"this" => Some(Keyword::This),
            b"let" => Some(Keyword::Let),
            b"do" => Some(Keyword::Do),
            b"if" => Some(Keyword::If),
            b"else" => Some(Keyword::Else),
            b"while" => Some(Keyword::While),
            b"return" => Some(Keyword::Return),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Keyword::Class => "class",
            Keyword::Constructor => "constructor", 
            Keyword::Function => "function",
            Keyword::Method => "method",
            Keyword::Field => "field", 
            Keyword::Static => "static", 
            Keyword::Var => "var", 
            Keyword::Int => "int", 
            Keyword::Char => "char", 
            Keyword::Boolean => "boolean", 
            Keyword::Void => "void", 
            Keyword::True => "true", 
            Keyword::False => "false", 
            Keyword::Null => "null", 
            Keyword::This => "this", 
            Keyword::Let => "let", 
            Keyword::Do => "do", 
            Keyword::If => "if", 
            Keyword::Else => "else", 
            Keyword::While => "while", 
            Keyword::Return => "return", 
        }
    } 
}

impl<'a> TryFrom<&'a [u8]> for Keyword {
    type Error = &'a [u8];
    fn try_from(value: &[u8]) -> Result<Self, &[u8]> {
        Keyword::new(value).ok_or(value) 
    }
}
