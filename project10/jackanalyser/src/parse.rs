use std::{fmt::Display, io::Write};

use crate::lexer::{Lexer, Keyword, Token};

pub fn parse(lexer: &mut Lexer) {
    class(lexer, std::io::stdout()).unwrap(); 
}

fn class<'a>(lexer: &mut Lexer<'a>, out: impl Write) -> Result<(), Error<'a>> {
    println!("<class>");
    ensure_tok(Token::Keyword(Keyword::Class), lexer)?;
    ident(lexer)?;
    ensure_tok(Token::Symbol(b'{'), lexer)?;

    loop {
        match lexer.peek() {
            Some(Token::Keyword(Keyword::Static | Keyword::Field)) => class_var_decl(lexer),
            Some(Token::Keyword(Keyword::Constructor | Keyword::Function | Keyword::Method)) => subroutine_decl(lexer),
            Some(Token::Symbol(b'}')) => break,
            Some(tok) => Err(BadToken(tok)),
            None => Err(EOF),
        }?;
    }
    
    ensure_tok(Token::Symbol(b'}'), lexer)?;

    println!("</class>");
    Ok(())
}

fn subroutine_decl<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<subroutineDec>");
    match lexer.peek() {
        Some(Token::Keyword(Keyword::Constructor | Keyword::Function | Keyword::Method)) => keyword(lexer),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }?;

    match lexer.peek() {
        Some(Token::Keyword(Keyword::Void)) => keyword(lexer),
        _ => ty(lexer),
    }?;

    ident(lexer)?;

    ensure_tok(Token::Symbol(b'('), lexer)?;
    parameter_list(lexer)?;
    ensure_tok(Token::Symbol(b')'), lexer)?;
    
    subroutine_body(lexer)?;

    println!("</subroutineDec>");
    Ok(())
}

fn subroutine_body<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<subroutineBody>");
    ensure_tok(Token::Symbol(b'{'), lexer)?;
    
    while lexer.peek() == Some(Token::Keyword(Keyword::Var)) {
        var_dec(lexer)?; 
    }

    statements(lexer)?;

    ensure_tok(Token::Symbol(b'}'), lexer)?;
    println!("</subroutineBody>");
    Ok(())
}

fn var_dec<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<varDec>");
    ensure_tok(Token::Keyword(Keyword::Var), lexer)?;
    ty(lexer)?;
    ident(lexer)?;

    while lexer.peek() == Some(Token::Symbol(b',')) {
        ensure_tok(Token::Symbol(b','), lexer)?;
        ident(lexer)?;
    }

    ensure_tok(Token::Symbol(b';'), lexer)?;
    println!("</varDec>");
    Ok(())
}

fn parameter_list<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<parameterList>");
    match ty(lexer) {
        Ok(()) => {
            ident(lexer)?;
            while lexer.peek() == Some(Token::Symbol(b',')) {
                ensure_tok(Token::Symbol(b','), lexer)?;
                ty(lexer)?;
                ident(lexer)?;
            }
        }
        Err(BadToken(_)) => {},
        Err(EOF) => return Err(EOF),
    } 
    println!("</parameterList>");
    Ok(())
}

fn class_var_decl<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<classVarDec>");
    match lexer.next() {
        Some(Token::Keyword(k @ (Keyword::Static | Keyword::Field))) => simple_tag("keyword", k.as_str()),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }?;

    ty(lexer)?;

    ident(lexer)?;

    while lexer.peek() == Some(Token::Symbol(b',')) {
        ensure_tok(Token::Symbol(b','), lexer)?;
        ident(lexer)?;
    }

    ensure_tok(Token::Symbol(b';'), lexer)?;
    
    println!("</classVarDec>");
    Ok(())
}

fn ty<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    match lexer.peek() {
        Some(Token::Keyword(Keyword::Int | Keyword::Char | Keyword::Boolean)) => keyword(lexer),
        Some(Token::Ident(_)) => ident(lexer),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF)
    }
}

fn statement<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    use Keyword::*;
    match lexer.peek() {
        Some(Token::Keyword(Let)) => let_statement(lexer),
        Some(Token::Keyword(If)) => if_statement(lexer),
        Some(Token::Keyword(While)) => while_statement(lexer),
        Some(Token::Keyword(Do)) => do_statement(lexer),
        Some(Token::Keyword(Return)) => return_statement(lexer),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }
}

fn return_statement<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<returnStatement>");
    ensure_tok(Token::Keyword(Keyword::Return), lexer)?;
    if lexer.peek() != Some(Token::Symbol(b';')) {
        expression(lexer)?;
    }
    ensure_tok(Token::Symbol(b';'), lexer)?;
    println!("</returnStatement>");
    Ok(())
}

fn do_statement<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<doStatement>");
    ensure_tok(Token::Keyword(Keyword::Do), lexer)?;
    subroutine_call(lexer)?;
    ensure_tok(Token::Symbol(b';'), lexer)?;
    println!("</doStatement>");
    Ok(())
}

fn while_statement<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<whileStatement>");
    ensure_tok(Token::Keyword(Keyword::While), lexer)?;
    ensure_tok(Token::Symbol(b'('), lexer)?;
    expression(lexer)?;
    ensure_tok(Token::Symbol(b')'), lexer)?;

    ensure_tok(Token::Symbol(b'{'), lexer)?;
    statements(lexer)?; 
    ensure_tok(Token::Symbol(b'}'), lexer)?;

    println!("</whileStatement>");
    Ok(())
}

fn if_statement<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<ifStatement>");
    ensure_tok(Token::Keyword(Keyword::If), lexer)?;
    ensure_tok(Token::Symbol(b'('), lexer)?;
    expression(lexer)?;
    ensure_tok(Token::Symbol(b')'), lexer)?;

    ensure_tok(Token::Symbol(b'{'), lexer)?;
    statements(lexer)?;
    ensure_tok(Token::Symbol(b'}'), lexer)?;
    
    if lexer.peek() == Some(Token::Keyword(Keyword::Else)) {
        simple_tag("keyword", "else")?;
        lexer.next();
        ensure_tok(Token::Symbol(b'{'), lexer)?;
        statements(lexer)?;
        ensure_tok(Token::Symbol(b'}'), lexer)?;
    }

    println!("</ifStatement>");
    Ok(())
}

fn let_statement<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<letStatement>");
    ensure_tok(Token::Keyword(Keyword::Let), lexer)?;

    ident(lexer)?;
    if lexer.peek() == Some(Token::Symbol(b'[')) {
        ensure_tok(Token::Symbol(b'['), lexer)?;
        expression(lexer)?;
        ensure_tok(Token::Symbol(b']'), lexer)?;
    }

    ensure_tok(Token::Symbol(b'='), lexer)?;

    expression(lexer)?;

    ensure_tok(Token::Symbol(b';'), lexer)?;

    println!("</letStatement>");
    Ok(())
}

fn expression<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<expression>");
    term(lexer)?;
    while let Some(Token::Symbol(c)) = lexer.peek() {
        if !is_binary_op(c) {
            break;
        }
        lexer.next().unwrap().print_tag();
        term(lexer)?;
    }
    println!("</expression>");
    Ok(())
}

fn subroutine_call<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    ident(lexer)?;
    match lexer.next() {
        Some(Token::Symbol(b'(')) => {
            simple_tag("symbol", '(')?;
            expression_list(lexer)?;
            ensure_tok(Token::Symbol(b')'), lexer)?;
            Ok(())
        }
        Some(Token::Symbol(b'.')) => {
            simple_tag("symbol", '.')?;
            ident(lexer)?;
            ensure_tok(Token::Symbol(b'('), lexer)?;
            expression_list(lexer)?;
            ensure_tok(Token::Symbol(b')'), lexer)?;
            Ok(())
        }
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }
}

fn expression_list<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<expressionList>");
    if lexer.peek() != Some(Token::Symbol(b')')) && lexer.peek() != None {
        expression(lexer)?;
    }
    while lexer.peek() == Some(Token::Symbol(b',')) {
        ensure_tok(Token::Symbol(b','), lexer)?;
        expression(lexer)?;
    }
    println!("</expressionList>");
    Ok(())
}

fn statements<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<statements>");
    while lexer.peek() != Some(Token::Symbol(b'}')) {
        statement(lexer)?;
    }
    println!("</statements>");
    Ok(())
}

fn is_binary_op(c: u8) -> bool {
    match c {
        b'+' | b'-' | b'*' | b'/' | b'&' | b'|' | b'<' | b'>' | b'=' => true,
        _ => false,
    }
}

fn simple_tag(name: &str, v: impl Display) -> Result<(), Error<'_>> {
    println!("<{name}> {v} </{name}>");
    Ok(())
}

fn term<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    println!("<term>");
    match lexer.peek() {
        Some(token) => match &token {
            Token::IntConstant(v) => { lexer.next(); simple_tag("integerConstant", v) },
            Token::StringConstant(v) => simple_tag("stringConstant", unsafe { lexer.next(); std::str::from_utf8_unchecked(v)  }),
            Token::Keyword(k) => { lexer.next(); match k { Keyword::True | Keyword::False | Keyword::Null | Keyword::This => simple_tag("keyword", k.as_str()),
                _ => Err(BadToken(token))} },
            Token::Ident(_) => match { let mut lexer = lexer.clone(); lexer.next(); lexer.next() } {
                Some(Token::Symbol(b'(') | Token::Symbol(b'.')) => subroutine_call(lexer),
                Some(t @ Token::Symbol(b'[')) => {
                    ident(lexer)?;
                    lexer.next();
                    t.print_tag();
                    expression(lexer)?;
                    ensure_tok(Token::Symbol(b']'), lexer)
                },
                Some(_) => ident(lexer),
                None => Err(EOF),
            },
            Token::Symbol(b'(') => {
                ensure_tok(Token::Symbol(b'('), lexer)?;
                expression(lexer)?;
                ensure_tok(Token::Symbol(b')'), lexer)
            }
            op @ Token::Symbol(b'-' | b'~') => {
                lexer.next();
                op.print_tag();
                term(lexer)
            }
            _ => Err(BadToken(token)),
        }
        None => Err(EOF),
    }?;
    println!("</term>");
    Ok(())
}

fn ident<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    let tok = lexer.next();
    match tok {
        Some(Token::Ident(s)) => simple_tag("identifier", unsafe { std::str::from_utf8_unchecked(s) }),
        Some(x) => Err(BadToken(x)),
        None => Err(EOF),
    }
}

fn keyword<'a>(lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    let tok = lexer.next();
    match tok {
        Some(Token::Keyword(k)) => simple_tag("keyword", k.as_str()),
        Some(x) => Err(BadToken(x)),
        None => Err(EOF),
    }
}


fn ensure_tok<'a>(token: Token<'a>, lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    match lexer.next() {
        Some(t) => if token == t {
            t.print_tag();
            Ok(())
        } else {
            Err(BadToken(t))
            },
        None => Err(EOF),
    }
}

impl<'a> Token<'a> {
    fn print_tag(&self) {
        match self {
            Token::IntConstant(i) => simple_tag("integerConstant", i),
            Token::StringConstant(s) => simple_tag("stringConstant", unsafe { std::str::from_utf8_unchecked(s) } ),
            Token::Ident(s) => simple_tag("identifier", unsafe { std::str::from_utf8_unchecked(s) } ),
            Token::Keyword(k) => simple_tag("keyword", k.as_str()),
            Token::Symbol(c) => match c { 
                b'>' => simple_tag("symbol", "&gt;"),
                b'<' => simple_tag("symbol", "&lt;"),
                b'"' => simple_tag("symbol", "&quot;"),
                b'&' => simple_tag("symbol", "&amp;"),
                c => simple_tag("symbol", *c as char),
            }
        }.unwrap();

    }
}

use Error::{EOF, BadToken};
pub enum Error<'a> {
    EOF,
    BadToken(Token<'a>),
}

impl<'a> std::error::Error for Error<'a> {

}

impl<'a> std::fmt::Debug for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EOF => write!(f, "Unexpected EOF"),
            Self::BadToken(arg0) => write!(f, "Unexpected token: {arg0:?}"),
        }
    }
}

impl<'a> std::fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EOF => write!(f, "Unexpected EOF"),
            Self::BadToken(arg0) => write!(f, "Unexpected token: {arg0:?}"),
        }
    }
}
