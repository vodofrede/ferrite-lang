use crate::{error, Error, Kind, Token, Tokens};
use std::iter::Peekable;

pub fn parse(tokens: Tokens) -> Result<Expr, Error> {
    let mut tokens = tokens.peekable();
    let root = block(&mut tokens, None).inspect_err(|e| eprintln!("parsing error: {e}"))?;
    ty(&root, &mut Env::new()).inspect_err(|e| eprintln!("type error: {e}"))?;
    Ok(root)
}

fn primary(tokens: &mut Peekable<Tokens>) -> Result<Expr, Error> {
    let token = tokens.next().ok_or(error!("unexpected eof"))?;
    let expr = match token.kind {
        Kind::Number => Expr::Number(token.text.parse().unwrap()),
        Kind::Text => Expr::Text(token.text.to_string()),
        Kind::Bool => Expr::Bool(token.text.parse().unwrap()),
        Kind::Identifier => Expr::Identifier(token.text.to_string()),
        Kind::Operator => {
            let power = prefix_power(token.text)
                .ok_or(error!("{:?} is not a unary operator", token.text).span(token.span))?;
            let right = expression(tokens, power)?;
            Expr::Unary {
                op: token.text.to_string(),
                expr: Box::new(right),
            }
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
                Expr::Variable(Box::new(left), Box::new(right))
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
    Ok(expr)
}
fn expression(tokens: &mut Peekable<Tokens>, min_power: usize) -> Result<Expr, Error> {
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
        left = Expr::Binary {
            op: op.to_string(),
            lhs: Box::new(left),
            rhs: Box::new(right),
        };
    }

    Ok(left)
}
fn block(tokens: &mut Peekable<Tokens>, end: Option<&str>) -> Result<Expr, Error> {
    let mut es = vec![];
    while let Some(token) = tokens.peek() {
        if end.is_some_and(|text| text == token.text) {
            break;
        }
        es.push(expression(tokens, 0)?);
    }
    Ok(Expr::Block(es))
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

#[derive(Debug, Clone)]
pub enum Expr {
    Bool(bool),
    Number(f64),
    Text(String),
    Identifier(String),
    List(Vec<Expr>),
    Seq(Vec<Expr>),
    Table(),
    Pattern(),
    Block(Vec<Expr>),
    Unary {
        op: String,
        expr: Box<Expr>,
    },
    Binary {
        op: String,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        yes: Box<Expr>,
        no: Box<Expr>,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<(Expr, Expr)>,
    },
    Loop {
        body: Box<Expr>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Expr>,
    },
    For {
        id: String,
        iter: Box<Expr>,
        body: Box<Expr>,
    },
    // expression statements
    Variable(Box<Expr>, Box<Expr>),
    Break,
    Return(Pattern),
    Function {},
    Type {},
    Record {},
    Trait {},
}
#[derive(Debug, Clone)]
pub enum Pattern {}

type Env = std::collections::HashMap<String, String>;
pub fn ty(expr: &Expr, env: &mut Env) -> Result<String, Error> {
    let ty = match expr {
        Expr::Bool(_) => String::from("bool"),
        Expr::Number(_) => String::from("number"),
        Expr::Text(_) => String::from("text"),
        Expr::Identifier(id) => env.get(id).ok_or(error!("unbound identifier"))?.clone(),
        Expr::List(es) => {
            let types = es
                .iter()
                .map(|p| ty(p, env))
                .collect::<Result<Vec<_>, _>>()?;
            types
                .windows(2)
                .all(|p| p[0] == p[1])
                .then_some(types.get(0).cloned().unwrap_or("unit".to_string()))
                .ok_or(error!("list is not homogenous"))?
        }
        Expr::Seq(es) => todo!(),
        Expr::Table() => todo!(),
        Expr::Unary { expr, .. } => ty(&expr, env)?,
        Expr::Binary { op, lhs, rhs } => {
            if let ("=", Expr::Identifier(a)) = (op.as_str(), lhs.as_ref()) {
                let t = ty(&rhs, env)?;
                env.insert(a.to_string(), t);
            }
            match (ty(&lhs, env)?, ty(&rhs, env)?) {
                (a, b) if a == b => a,
                (a, b) => return Err(error!("expected {a}, found {b}")),
            }
        }
        Expr::Pattern() => todo!(),
        Expr::Block(b) => {
            let mut env = env.clone();
            b.iter()
                .map(|e| ty(e, &mut env))
                .last()
                .unwrap_or(Ok("unit".to_string()))?
        }
        Expr::Variable(_, _) => todo!(),
        Expr::If { cond, yes, no } => {
            match (ty(&cond, env)?.as_str(), ty(&yes, env)?, ty(&no, env)?) {
                ("bool", a, b) if a == b => a,
                ("bool", a, b) if a != b => return Err(error!("expected {a}, found {b}")),
                (c, _a, _b) => return Err(error!("expected bool, found {c}")),
            }
        }
        Expr::Match { expr, arms } => "unit".to_string(),
        Expr::Loop { body } => todo!(),
        Expr::While { cond, body } => todo!(),
        Expr::For { id, iter, body } => todo!(),
        Expr::Break => todo!(),
        Expr::Return(_) => todo!(),
        Expr::Function {} => todo!(),
        Expr::Type {} => todo!(),
        Expr::Record {} => todo!(),
        Expr::Trait {} => todo!(),
    };

    Ok(ty)
}
