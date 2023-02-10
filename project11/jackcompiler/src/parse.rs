use std::io::Write;

use crate::{
    codegen::{self, CodeGen},
    lexer::{Keyword, Lexer, Token},
};

// Parses a string which contains a single class
pub fn parse(lexer: &mut Lexer, out: impl Write) {
    let mut codegen = CodeGen::new(out);
    if let Err(e) = class(lexer, &mut codegen) {
        eprintln!("{:?}", e);
        eprintln!(
            "line: {:?}",
            lexer.text[0..lexer.idx]
                .iter()
                .fold(0, |acc, ch| if *ch == b'\n' { acc + 1 } else { acc })
                + 1
        );
    }
}

fn class<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    ensure_tok(Token::Keyword(Keyword::Class), lexer)?;
    let name = ident(lexer)?;
    codegen.class_name = name.to_owned();
    ensure_tok(Token::Symbol(b'{'), lexer)?;

    while let Some(Token::Keyword(Keyword::Static | Keyword::Field)) = lexer.peek() {
        class_var_decl(lexer, codegen)?;
    }

    while let Some(Token::Keyword(Keyword::Constructor | Keyword::Function | Keyword::Method)) =
        lexer.peek()
    {
        subroutine_decl(lexer, codegen, codegen::Ty::Class(name))?
    }
    ensure_tok(Token::Symbol(b'}'), lexer)?;

    Ok(())
}

fn subroutine_decl<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
    class_ty: codegen::Ty<'a>,
) -> Result<(), Error<'a>> {
    codegen.reset_subroutine();
    let subroutine_type = match lexer.peek() {
        Some(Token::Keyword(Keyword::Method)) => {
            codegen.add_symbol(b"this", class_ty, codegen::Kind::Argument);
            keyword(lexer)
        }
        Some(Token::Keyword(Keyword::Constructor | Keyword::Function)) => keyword(lexer),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }?;

    match lexer.peek() {
        Some(Token::Keyword(Keyword::Void)) => {
            keyword(lexer)?;
        }
        _ => {
            ty(lexer)?;
        }
    };

    let name = ident(lexer)?;

    ensure_tok(Token::Symbol(b'('), lexer)?;
    parameter_list(lexer, codegen)?;
    ensure_tok(Token::Symbol(b')'), lexer)?;
    let name = codegen
        .class_name()
        .iter()
        .chain(b".")
        .chain(name.iter())
        .copied()
        .collect::<Vec<u8>>();

    subroutine_body(lexer, &name, subroutine_type, codegen)?;

    Ok(())
}

fn subroutine_body<'a>(
    lexer: &mut Lexer<'a>,
    func_name: &[u8],
    subroutine_type: Keyword,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    ensure_tok(Token::Symbol(b'{'), lexer)?;

    let mut vars_count = 0;
    while lexer.peek() == Some(Token::Keyword(Keyword::Var)) {
        vars_count += var_dec(lexer, codegen)?;
    }

    codegen.function(func_name, vars_count);
    if subroutine_type == Keyword::Method {
        codegen.push(codegen::Segment::Argument, 0);
        codegen.pop(codegen::Segment::Pointer, 0);
    } else if subroutine_type == Keyword::Constructor {
        codegen.push(codegen::Segment::Constant, codegen.fields_count());
        codegen.call(b"Memory.alloc", 1);
        codegen.pop(codegen::Segment::Pointer, 0);
    }

    statements(lexer, codegen)?;

    ensure_tok(Token::Symbol(b'}'), lexer)?;
    Ok(())
}

fn var_dec<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<u16, Error<'a>> {
    ensure_tok(Token::Keyword(Keyword::Var), lexer)?;
    let ty = ty(lexer)?;
    let name = ident(lexer)?;
    codegen.add_symbol(name, ty, codegen::Kind::Local);
    let mut var_count = 1;
    while lexer.peek() == Some(Token::Symbol(b',')) {
        ensure_tok(Token::Symbol(b','), lexer)?;
        let name = ident(lexer)?;
        codegen.add_symbol(name, ty, codegen::Kind::Local);
        var_count += 1;
    }

    ensure_tok(Token::Symbol(b';'), lexer)?;
    Ok(var_count)
}

fn parameter_list<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<u16, Error<'a>> {
    match ty(lexer) {
        Ok(first_ty) => {
            let mut count = 1;
            let arg = ident(lexer)?;
            codegen.add_symbol(arg, first_ty, codegen::Kind::Argument);
            while lexer.peek() == Some(Token::Symbol(b',')) {
                ensure_tok(Token::Symbol(b','), lexer)?;
                let ty = ty(lexer)?;
                let arg = ident(lexer)?;
                codegen.add_symbol(arg, ty, codegen::Kind::Argument);
                count += 1;
            }
            Ok(count)
        }
        Err(BadToken(Token::Symbol(b')'))) => Ok(0),
        Err(e) => Err(e),
    }
}

fn class_var_decl<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    let k = match lexer.next() {
        Some(Token::Keyword(k @ (Keyword::Static | Keyword::Field))) => Ok(k),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }?
    .try_into()
    .unwrap();

    let ty = ty(lexer)?;

    let name = {
        let tok = lexer.next();
        match tok {
            Some(Token::Ident(s)) => Ok(s),
            Some(x) => Err(BadToken(x)),
            None => Err(EOF),
        }
    }?;

    codegen.add_symbol(name, ty, k);

    while lexer.peek() == Some(Token::Symbol(b',')) {
        ensure_tok(Token::Symbol(b','), lexer)?;
        let name = {
            let tok = lexer.next();
            match tok {
                Some(Token::Ident(s)) => Ok(s),
                Some(x) => Err(BadToken(x)),
                None => Err(EOF),
            }
        }?;

        codegen.add_symbol(name, ty, k);
    }

    ensure_tok(Token::Symbol(b';'), lexer)?;

    Ok(())
}

fn ty<'a>(lexer: &mut Lexer<'a>) -> Result<codegen::Ty<'a>, Error<'a>> {
    match lexer.peek() {
        Some(Token::Keyword(Keyword::Int | Keyword::Char | Keyword::Boolean)) => {
            Ok(codegen::Ty::from_token(lexer.next().unwrap()))
        }
        Some(Token::Ident(_)) => Ok(codegen::Ty::from_token(lexer.next().unwrap())),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }
}

fn statement<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    use Keyword::*;
    match lexer.peek() {
        Some(Token::Keyword(Let)) => let_statement(lexer, codegen),
        Some(Token::Keyword(If)) => if_statement(lexer, codegen),
        Some(Token::Keyword(While)) => while_statement(lexer, codegen),
        Some(Token::Keyword(Do)) => do_statement(lexer, codegen),
        Some(Token::Keyword(Return)) => return_statement(lexer, codegen),
        Some(t) => Err(BadToken(t)),
        None => Err(EOF),
    }
}

fn return_statement<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    ensure_tok(Token::Keyword(Keyword::Return), lexer)?;
    if lexer.peek() != Some(Token::Symbol(b';')) {
        expression(lexer, codegen)?;
    }
    codegen.write_return();
    ensure_tok(Token::Symbol(b';'), lexer)?;
    Ok(())
}

fn do_statement<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    ensure_tok(Token::Keyword(Keyword::Do), lexer)?;
    subroutine_call(lexer, codegen)?;
    codegen.pop(codegen::Segment::Temp, 0);
    ensure_tok(Token::Symbol(b';'), lexer)?;
    Ok(())
}

fn while_statement<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    use std::iter;
    let loop_label = codegen.next_label_idx();
    let loop_label = b"L"
        .iter()
        .copied()
        .chain(iter::once(loop_label as u8 + b'0'))
        .collect::<Vec<u8>>();
    codegen.label(&loop_label);

    ensure_tok(Token::Keyword(Keyword::While), lexer)?;
    ensure_tok(Token::Symbol(b'('), lexer)?;
    expression(lexer, codegen)?;
    ensure_tok(Token::Symbol(b')'), lexer)?;
    codegen.arithmetic(codegen::ArithmeticInstruction::Not);
    let break_label = codegen.next_label_idx();
    let break_label = b"L"
        .iter()
        .copied()
        .chain(iter::once(break_label as u8 + b'0'))
        .collect::<Vec<u8>>();
    codegen.if_goto(&break_label);

    ensure_tok(Token::Symbol(b'{'), lexer)?;
    statements(lexer, codegen)?;
    ensure_tok(Token::Symbol(b'}'), lexer)?;
    codegen.goto(&loop_label);

    codegen.label(&break_label);
    Ok(())
}

fn if_statement<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    use std::iter;
    ensure_tok(Token::Keyword(Keyword::If), lexer)?;
    ensure_tok(Token::Symbol(b'('), lexer)?;
    // condition
    expression(lexer, codegen)?;
    codegen.arithmetic(codegen::ArithmeticInstruction::Not);
    let label_over_if = codegen.next_label_idx();
    let label_over_if = b"L"
        .iter()
        .copied()
        .chain(iter::once(label_over_if as u8 + b'0'));
    let label_over_if = label_over_if.collect::<Vec<u8>>();
    codegen.if_goto(&label_over_if);
    ensure_tok(Token::Symbol(b')'), lexer)?;

    ensure_tok(Token::Symbol(b'{'), lexer)?;
    statements(lexer, codegen)?;
    let label_end = codegen.next_label_idx();
    let label_end = b"L"
        .iter()
        .copied()
        .chain(iter::once(label_end as u8 + b'0'))
        .collect::<Vec<u8>>();
    codegen.goto(&label_end);
    ensure_tok(Token::Symbol(b'}'), lexer)?;

    codegen.label(&label_over_if);
    if lexer.peek() == Some(Token::Keyword(Keyword::Else)) {
        lexer.next();
        ensure_tok(Token::Symbol(b'{'), lexer)?;
        statements(lexer, codegen)?;
        ensure_tok(Token::Symbol(b'}'), lexer)?;
    }
    codegen.label(&label_end);

    Ok(())
}

fn let_statement<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    ensure_tok(Token::Keyword(Keyword::Let), lexer)?;

    let name = ident(lexer)?;
    let is_array = if lexer.peek() == Some(Token::Symbol(b'[')) {
        ensure_tok(Token::Symbol(b'['), lexer)?;
        expression(lexer, codegen)?;
        let symbol = codegen.get_symbol(name).unwrap();
        codegen.push(symbol.kind.into(), symbol.idx);
        codegen.arithmetic(codegen::ArithmeticInstruction::Add);
        ensure_tok(Token::Symbol(b']'), lexer)?;
        true
    } else {
        false
    };

    ensure_tok(Token::Symbol(b'='), lexer)?;

    expression(lexer, codegen)?;

    if is_array {
        codegen.pop(codegen::Segment::Temp, 0);
        codegen.pop(codegen::Segment::Pointer, 1);
        codegen.push(codegen::Segment::Temp, 0);
        codegen.pop(codegen::Segment::That, 0);
    } else {
        let symbol = codegen.get_symbol(name).unwrap();
        codegen.pop(symbol.kind.into(), symbol.idx);
    }

    ensure_tok(Token::Symbol(b';'), lexer)?;

    Ok(())
}

fn expression<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    term(lexer, codegen)?;
    while let Some(Token::Symbol(c)) = lexer.peek() {
        if !is_binary_op(c) {
            break;
        }
        lexer.next();
        term(lexer, codegen)?;
        use codegen::ArithmeticInstruction::*;
        match c {
            b'+' => codegen.arithmetic(Add),
            b'-' => codegen.arithmetic(Sub),
            b'*' => codegen.call(b"Math.multiply", 2),
            b'/' => codegen.call(b"Math.divide", 2),
            b'|' => codegen.arithmetic(Or),
            b'&' => codegen.arithmetic(And),
            b'<' => codegen.arithmetic(Lt),
            b'>' => codegen.arithmetic(Gt),
            b'=' => codegen.arithmetic(Eq),
            _ => unreachable!(),
        }
    }
    Ok(())
}

fn subroutine_call<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    let name = ident(lexer)?;
    let obj = codegen.get_symbol(name).cloned();

    match lexer.peek() {
        Some(Token::Symbol(b'.')) => match obj {
            Some(obj) => {
                codegen.push(obj.kind.into(), obj.idx);
                let class_name = match obj.ty {
                    codegen::Ty::Class(cn) => cn,
                    _ => panic!("ATTEMPT TO USE METHOD ON PRIMITIVE"),
                };
                ensure_tok(Token::Symbol(b'.'), lexer)?;
                let func_name = ident(lexer)?;
                let func_name = class_name
                    .iter()
                    .chain(b".".iter())
                    .chain(func_name.iter())
                    .copied()
                    .collect::<Vec<u8>>();
                ensure_tok(Token::Symbol(b'('), lexer)?;
                let args_count = expression_list(lexer, codegen)?;
                ensure_tok(Token::Symbol(b')'), lexer)?;
                codegen.call(&func_name, args_count + 1);
                Ok(())
            }
            None => {
                ensure_tok(Token::Symbol(b'.'), lexer)?;
                let func_name = ident(lexer)?;
                let func_name = name
                    .iter()
                    .chain(b".".iter())
                    .chain(func_name.iter())
                    .copied()
                    .collect::<Vec<u8>>();
                ensure_tok(Token::Symbol(b'('), lexer)?;
                let args_count = expression_list(lexer, codegen)?;
                codegen.call(&func_name, args_count);
                ensure_tok(Token::Symbol(b')'), lexer)
            }
        },
        Some(Token::Symbol(b'(')) => {
            ensure_tok(Token::Symbol(b'('), lexer)?;
            let func_name = codegen
                .class_name()
                .iter()
                .chain(b".".iter())
                .chain(name.iter())
                .copied()
                .collect::<Vec<u8>>();
            codegen.push(codegen::Segment::Pointer, 0);
            let args_count = expression_list(lexer, codegen)?;
            codegen.call(&func_name, args_count + 1);
            ensure_tok(Token::Symbol(b')'), lexer)
        }
        Some(tok) => Err(BadToken(tok)),
        None => Err(EOF),
    }?;

    Ok(())
}

fn expression_list<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<u16, Error<'a>> {
    let mut count = 0;
    if lexer.peek() != Some(Token::Symbol(b')')) && lexer.peek().is_some() {
        expression(lexer, codegen)?;
        count += 1;
        while lexer.peek() == Some(Token::Symbol(b',')) {
            count += 1;
            ensure_tok(Token::Symbol(b','), lexer)?;
            expression(lexer, codegen)?;
        }
    }

    Ok(count)
}

fn statements<'a>(
    lexer: &mut Lexer<'a>,
    codegen: &mut CodeGen<'a, impl Write>,
) -> Result<(), Error<'a>> {
    while lexer.peek() != Some(Token::Symbol(b'}')) {
        statement(lexer, codegen)?;
    }
    Ok(())
}

fn is_binary_op(c: u8) -> bool {
    matches!(
        c,
        b'+' | b'-' | b'*' | b'/' | b'&' | b'|' | b'<' | b'>' | b'='
    )
}

fn term<'a>(lexer: &mut Lexer<'a>, codegen: &mut CodeGen<'a, impl Write>) -> Result<(), Error<'a>> {
    use codegen::{ArithmeticInstruction, Segment};
    match lexer.peek() {
        Some(token) => match &token {
            Token::IntConstant(v) => {
                lexer.next();
                codegen.push(Segment::Constant, *v);
                Ok(())
            }
            Token::StringConstant(v) => {
                lexer.next();
                // TODO: strings
                codegen.push(Segment::Constant, (v.len() + 1) as u16);
                codegen.call(b"String.new", 1);
                for c in v.iter() {
                    codegen.push(Segment::Constant, *c as u16);
                    codegen.call(b"String.appendChar", 2);
                }
                Ok(())
            }
            Token::Keyword(k) => {
                lexer.next();
                match k {
                    Keyword::True => {
                        codegen.push(Segment::Constant, 1);
                        codegen.arithmetic(ArithmeticInstruction::Neg);
                        Ok(())
                    }
                    Keyword::False | Keyword::Null => {
                        codegen.push(Segment::Constant, 0);
                        Ok(())
                    }
                    Keyword::This => {
                        codegen.push(Segment::Pointer, 0);
                        Ok(())
                    }
                    _ => Err(BadToken(token)),
                }
            }
            Token::Ident(_) => match {
                let mut lexer = lexer.clone();
                lexer.next();
                lexer.next()
            } {
                Some(Token::Symbol(b'(' | b'.')) => subroutine_call(lexer, codegen),
                Some(Token::Symbol(b'[')) => {
                    let array_name = ident(lexer)?;
                    ensure_tok(Token::Symbol(b'['), lexer)?;
                    // This leaves expression on the stack
                    expression(lexer, codegen)?;
                    let array_name = codegen.get_symbol(array_name).unwrap();
                    codegen.push(array_name.kind.into(), array_name.idx);
                    codegen.arithmetic(ArithmeticInstruction::Add);
                    codegen.pop(Segment::Pointer, 1);
                    codegen.push(Segment::That, 0);

                    ensure_tok(Token::Symbol(b']'), lexer)
                }
                Some(_) => {
                    let name = ident(lexer)?;
                    let variable = codegen.get_symbol(name).unwrap();
                    codegen.push(variable.kind.into(), variable.idx);
                    Ok(())
                }
                None => Err(EOF),
            },
            Token::Symbol(b'(') => {
                ensure_tok(Token::Symbol(b'('), lexer)?;
                expression(lexer, codegen)?;
                ensure_tok(Token::Symbol(b')'), lexer)
            }
            Token::Symbol(op @ (b'-' | b'~')) => {
                lexer.next();
                term(lexer, codegen)?;
                match op {
                    b'-' => codegen.arithmetic(ArithmeticInstruction::Neg),
                    b'~' => codegen.arithmetic(ArithmeticInstruction::Not),
                    _ => {}
                }
                Ok(())
            }
            _ => Err(BadToken(token)),
        },
        None => Err(EOF),
    }?;
    Ok(())
}

fn ident<'a>(lexer: &mut Lexer<'a>) -> Result<&'a [u8], Error<'a>> {
    let tok = lexer.next();
    match tok {
        Some(Token::Ident(s)) => Ok(s),
        Some(x) => Err(BadToken(x)),
        None => Err(EOF),
    }
}

fn keyword<'a>(lexer: &mut Lexer<'a>) -> Result<Keyword, Error<'a>> {
    let tok = lexer.next();
    match tok {
        Some(Token::Keyword(k)) => Ok(k),
        Some(x) => Err(BadToken(x)),
        None => Err(EOF),
    }
}

fn ensure_tok<'a>(token: Token<'a>, lexer: &mut Lexer<'a>) -> Result<(), Error<'a>> {
    match lexer.next() {
        Some(t) => {
            if token == t {
                Ok(())
            } else {
                Err(BadToken(t))
            }
        }
        None => Err(EOF),
    }
}
use Error::{BadToken, EOF};
pub enum Error<'a> {
    EOF,
    BadToken(Token<'a>),
}

impl<'a> std::error::Error for Error<'a> {}

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
