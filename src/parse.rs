use crate::{error, Error, Kind, Span, Token, Tokens};
use std::{fmt, iter::Peekable};

pub fn parse(tokens: Tokens) -> Result<Expr, Error> {
    let mut tokens = tokens.peekable();
    let root = Expr {
        item: block(&mut tokens, None).inspect_err(|e| eprintln!("parsing error: {e}"))?,
        span: Span { start: 0, len: 0 },
    };
    ty(&root, &mut Env::new()).inspect_err(|e| eprintln!("type error: {e}"))?;
    Ok(root)
}

fn primary<'a>(tokens: &mut Peekable<Tokens<'a>>) -> Result<Expr<'a>, Error> {
    let token = tokens.next().ok_or(error!("unexpected eof"))?;
    let item = match token.kind {
        Kind::Number => Item::Number(token.text.parse().unwrap()),
        Kind::Text => Item::Text(token.text),
        Kind::Bool => Item::Bool(token.text.parse().unwrap()),
        Kind::Identifier => Item::Identifier(token.text),
        Kind::Operator => {
            let power = prefix_power(token.text)
                .ok_or(error!("{:?} is not a unary operator", token.text).span(token.span))?;
            let right = expression(tokens, power)?;
            Item::Unary(token.text, Box::new(right))
        }
        Kind::Grouping => {
            // group -> '(' expr ')'
            let expr = expression(tokens, 0)?;
            expect(tokens, ")")?; // consume end grouping marker
            return Ok(expr);
        }
        Kind::Keyword => match token.text {
            "do" => {
                let block = block(tokens, Some("end"))?;
                expect(tokens, "end")?;
                block
            }
            "var" => {
                let left = expression(tokens, 0)?;
                expect(tokens, "=")?;
                let right = expression(tokens, 0)?;
                Item::Variable(Box::new(left), Box::new(right))
            }
            "if" => todo!(),
            "match" => todo!(),
            "loop" => todo!(),
            "while" => todo!(),
            "for" => todo!(),
            "function" => todo!(),
            "return" => todo!(),
            "type" => todo!(),
            "record" => todo!(),
            "trait" => todo!(),
            _ => return Err(error!("unexpected keyword {:?}", token.text).span(token.span)),
        },
        _ => return Err(error!("unexpected token {:?}", token.text).span(token.span)),
    };
    let span = token.span;
    Ok(Expr { item, span })
}
fn expression<'a>(tokens: &mut Peekable<Tokens<'a>>, min_power: usize) -> Result<Expr<'a>, Error> {
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

        let right = expression(tokens, right_power)?;
        let span = left.span.merge(right.span);
        left = Expr {
            item: Item::Binary(op, Box::new(left), Box::new(right)),
            span,
        };
    }

    Ok(left)
}
fn block<'a>(tokens: &mut Peekable<Tokens<'a>>, end: Option<&str>) -> Result<Item<'a>, Error> {
    let mut es = vec![];
    while let Some(token) = tokens.peek() {
        if end.map_or(false, |text| text == token.text) {
            break;
        }
        es.push(expression(tokens, 0)?);
    }
    Ok(Item::Block(es))
}
fn expect<'a>(tokens: &mut Peekable<Tokens<'a>>, text: &'a str) -> Result<Token<'a>, Error> {
    let token = tokens.next().ok_or(error!("unexpected eof"))?;
    if token.text != text {
        return Err(error!("expected {:?}, got {:?}", text, token.text));
    }
    Ok(token)
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
        "not" => 12,
        "-" => 12,
        _ => return None,
    };
    Some(power)
}

#[derive(Clone)]
pub struct Expr<'a> {
    pub item: Item<'a>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub enum Item<'a> {
    Bool(bool),
    Number(f64),
    Text(&'a str),
    Identifier(&'a str),
    Unary(&'a str, Box<Expr<'a>>),
    Binary(&'a str, Box<Expr<'a>>, Box<Expr<'a>>),
    Block(Vec<Expr<'a>>),
    Variable(Box<Expr<'a>>, Box<Expr<'a>>),
    If {
        expr: Box<Expr<'a>>,
        yes: Box<Expr<'a>>,
        no: Box<Expr<'a>>,
    },
    Match {
        expr: Box<Expr<'a>>,
        arms: Vec<(Expr<'a>, Expr<'a>)>,
    },
    Loop(Box<Expr<'a>>),
    While(Box<Expr<'a>>, Box<Expr<'a>>),
    For(Box<Expr<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    Function(),
    Return(),
    Type(),
    Record(),
    Trait(),
}

type Env<'a> = std::collections::HashMap<&'a str, &'a str>;
pub fn ty<'a>(expr: &'a Expr, env: &mut Env<'a>) -> Result<&'a str, Error> {
    let ty = match &expr.item {
        Item::Bool(_) => "bool",
        Item::Number(_) => "number",
        Item::Text(_) => "text",
        Item::Identifier(id) => env.get(id).ok_or(error!("unbound identifier"))?,
        Item::Unary(_, n) => ty(n, env)?,
        Item::Binary(op, first, second) => {
            if let ("=", Item::Identifier(a)) = (*op, first.item.clone()) {
                let t = ty(second, env)?;
                env.insert(a, t);
            }
            match (ty(first, env)?, ty(second, env)?) {
                (a, b) if a == b => a,
                (a, b) => return Err(error!("expected {a}, found {b}").span(second.span)),
            }
        }
        Item::Block(b) => {
            let mut env = env.clone();
            b.iter()
                .map(|e| ty(e, &mut env))
                .last()
                .unwrap_or(Ok("unit"))?
        }
        Item::Variable(_, _) => todo!(),
        Item::If(cond, then, other) => match (ty(cond, env)?, ty(then, env)?, ty(other, env)?) {
            ("bool", a, b) if a == b => a,
            ("bool", a, b) if a != b => {
                return Err(error!("expected {a}, found {b}").span(other.span))
            }
            (c, _a, _b) => return Err(error!("expected bool, found {c}").span(cond.span)),
        },
    };

    Ok(ty)
}

impl<'a> fmt::Debug for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stop = self.span.start + self.span.len;
        write!(f, "({:?} @ {}..{})", self.item, self.span.start, stop)
    }
}
