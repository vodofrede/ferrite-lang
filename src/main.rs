#![warn(clippy::all)]

mod ir;
mod parse;
mod token;
mod wasm;

use crate::{parse::*, token::*};
use std::{
    env, error, fmt, fs,
    io::{prelude::*, stdin, stdout},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    if let [_, file, ..] = args.as_slice() {
        let src = fs::read_to_string(file)?;
        let ast = parse(tokens(&src))?;
        let res = eval(ast, &mut Table::new()).inspect_err(|e| eprintln!("runtime error: {e}"))?;
        println!("{res:?}");
        // let ir = lower(ast)?;
        // let bin = dump(ir);
        // fs::write(Path::new(file).with_extension("wasm"), bin)?;
    } else {
        println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        let (stdin, mut stdout) = (stdin(), stdout());
        let mut lines = stdin.lock().lines();
        loop {
            print!("> ");
            stdout.flush()?;
            let src = lines.next().unwrap()?;
            let Ok(ast) = parse(tokens(&src)) else {
                continue;
            };
            let Ok(res) =
                eval(ast, &mut Table::new()).inspect_err(|e| eprintln!("runtime error: {e}"))
            else {
                continue;
            };
            println!("{res:?}");
        }
    }
    Ok(())
}

type Table<'a> = std::collections::HashMap<&'a str, Value>;
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Value {
    Unit,
    Bool(bool),
    Number(f64),
    Text(String),
}
fn eval<'a>(e: Expr<'a>, table: &mut Table<'a>) -> Result<Value, Error> {
    let value = match e.item {
        Item::Bool(b) => Value::Bool(b),
        Item::Number(n) => Value::Number(n),
        Item::Text(t) => Value::Text(t.to_string()),
        Item::Identifier(id) => table.get(id).cloned().ok_or(error!("unbound identifier"))?,
        Item::Unary(op, n) => match (op, n.item) {
            ("-", Item::Number(n)) => Value::Number(-n),
            ("not", Item::Bool(b)) => Value::Bool(!b),
            (_, n) => return Err(error!("cannot apply unary {op} to {:?}", n)),
        },
        Item::Binary(op, a, b) => {
            if let ("=", Item::Identifier(a), _) = (op, &a.item, &b) {
                let value = eval(*b, table)?;
                table.insert(a, value.clone());
                return Ok(value);
            }
            match (op, eval(*a, table)?, eval(*b, table)?) {
                ("+", Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                ("-", Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                ("*", Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                ("/", Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                ("^", Value::Number(a), Value::Number(b)) => Value::Number(a.powf(b)),
                ("==", Value::Number(a), Value::Number(b)) => Value::Bool(a == b),
                (">", Value::Number(a), Value::Number(b)) => Value::Bool(a > b),
                ("<", Value::Number(a), Value::Number(b)) => Value::Bool(a < b),
                (">=", Value::Number(a), Value::Number(b)) => Value::Bool(a >= b),
                ("<=", Value::Number(a), Value::Number(b)) => Value::Bool(a <= b),
                ("==", Value::Bool(a), Value::Bool(b)) => Value::Bool(a == b),
                ("and", Value::Bool(a), Value::Bool(b)) => Value::Bool(a && b),
                ("or", Value::Bool(a), Value::Bool(b)) => Value::Bool(a || b),
                (_, a, b) => return Err(error!("cannot apply binary {op} to {:?} and {:?}", a, b)),
            }
        }
        Item::Block(bl) => {
            let mut local = table.clone();
            bl.into_iter()
                .map(|e| eval(e, &mut local))
                .last()
                .unwrap_or(Ok(Value::Unit))?
        }
        Item::Variable(_, _) => todo!(),
        Item::Loop() => todo!(),
    };
    Ok(value)
}

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
    span: Option<Span>,
}
impl Error {
    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}
impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(span) = self.span {
            write!(f, "{} at index {}", self.msg, span.start)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::Error {
            msg: format!($($arg)*).replace('\n', "\\n"),
            span: None
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}
impl Span {
    pub fn merge(&self, other: Self) -> Self {
        let start = self.start.min(other.start);
        let end = (self.start + self.len).max(other.start + other.len);
        let len = end - start;
        Span { start, len }
    }
}
