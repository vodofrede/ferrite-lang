use crate::{error, Error, Kind, Token, Tokens};
use std::{fmt, iter::Peekable};

pub fn parse(tokens: Tokens) -> Result<Expr, Error> {
    let mut tokens = tokens.peekable();
    block(&mut tokens, None).inspect_err(|e| eprintln!("{e}"))
}

fn atom(token: Token) -> Result<Expr, Error> {
    let e = match token.kind {
        Kind::Number => Expr::Number(token.text.parse().unwrap()),
        Kind::Text => Expr::Text(token.text.to_string()),
        Kind::Bool => Expr::Bool(token.text.parse().unwrap()),
        Kind::Identifier => Expr::Identifier(token.text.to_string()),
        _ => return Err(error!("unexpected token {:?}", token.text).span(token.span)),
    };
    Ok(e)
}
fn primary(tokens: &mut Peekable<Tokens>) -> Result<Expr, Error> {
    let token = tokens.next().ok_or(error!("unexpected eof"))?;
    let e = match token.kind {
        Kind::Operator => {
            let power = prefix_power(token.text)
                .ok_or(error!("{:?} is not a unary operator", token.text).span(token.span))?;
            Expr::Unary(token.text.to_string(), Box::new(expr(tokens, power)?))
        }
        Kind::Separator => {
            let e = expr(tokens, 0)?;
            tokens.next();
            e
        }
        Kind::Keyword => match token.text {
            "do" => {
                let e = block(tokens, Some("end"))?;
                tokens.next();
                e
            } // do block
            _ => return Err(error!("unexpected keyword {:?}", token.text).span(token.span)),
        },
        _ => atom(token)?,
    };
    Ok(e)
}
fn expr(tokens: &mut Peekable<Tokens>, min_power: usize) -> Result<Expr, Error> {
    let mut left = primary(tokens)?;

    loop {
        let token = match tokens.peek() {
            Some(t) if t.kind == Kind::Operator => t,
            _ => break,
        };
        let op = token.text;

        let (left_power, right_power) =
            infix_power(op).ok_or(error!("{op:?} is not an infix operator").span(token.span))?;
        if left_power < min_power {
            break;
        }
        tokens.next().unwrap();
        let right = expr(tokens, right_power)?;
        left = Expr::Binary(op.to_string(), Box::new(left), Box::new(right));
    }

    Ok(left)
}
fn block(tokens: &mut Peekable<Tokens>, end: Option<&str>) -> Result<Expr, Error> {
    let mut es = vec![];
    while let Some(token) = tokens.peek() {
        if end.is_some_and(|t| token.text == t) {
            break;
        }
        let e = expr(tokens, 0)?;
        es.push(e);
    }
    Ok(Expr::Block(es))
}

fn infix_power(op: &str) -> Option<(usize, usize)> {
    let power = match op {
        "^" => (14, 15),
        "*" | "/" => (10, 11),
        "+" | "-" => (8, 9),
        "==" | "<" | ">" | "!=" => (6, 7),
        "and" => (4, 5),
        "or" => (2, 3),
        "=" => (1, 0),
        _ => return None,
    };
    Some(power)
}
fn prefix_power(op: &str) -> Option<usize> {
    let power = match op {
        "~" | "not" => 12,
        "-" => 12,
        _ => return None,
    };
    Some(power)
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Expr {
    Unit,
    Number(f64),
    Text(String),
    Bool(bool),
    Unary(String, Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
    Identifier(String),
    Block(Vec<Expr>),
}
impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => f.write_str("()"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Text(t) => write!(f, "{t:?}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Unary(op, a) => write!(f, "({op} {a:?})"),
            Self::Binary(op, a, b) => write!(f, "({a:?}{op}{b:?})"),
            Self::Identifier(id) => write!(f, "{id}"),
            Self::Block(bl) => write!(f, "{bl:?}"),
        }
    }
}
