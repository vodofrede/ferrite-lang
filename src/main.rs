#![warn(clippy::all)]

mod parse;
mod token;

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

type Table<'a> = std::collections::HashMap<String, Value>;
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Value {
    Unit,
    Bool(bool),
    Number(f64),
    Text(String),
}
fn eval(expr: Expr, table: &mut Table) -> Result<Value, Error> {
    let value = match expr {
        Expr::Bool(b) => Value::Bool(b),
        Expr::Number(n) => Value::Number(n),
        Expr::Text(t) => Value::Text(t.to_string()),
        Expr::Identifier(id) => table
            .get(&id)
            .cloned()
            .ok_or(error!("unbound identifier"))?,
        Expr::Unary { op, expr } => match (op.as_str(), *expr) {
            ("-", Expr::Number(n)) => Value::Number(-n),
            ("not", Expr::Bool(b)) => Value::Bool(!b),
            (_, n) => return Err(error!("cannot apply unary {op} to {:?}", n)),
        },
        Expr::Binary { op, lhs, rhs } => {
            if let ("=", Expr::Identifier(lhs)) = (op.as_str(), lhs.as_ref()) {
                let value = eval(*rhs, table)?;
                table.insert(lhs.clone(), value.clone());
                return Ok(value);
            }
            match (op.as_str(), eval(*lhs, table)?, eval(*rhs, table)?) {
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
        Expr::Block(bl) => {
            let mut local = table.clone();
            bl.into_iter()
                .map(|e| eval(e, &mut local))
                .last()
                .unwrap_or(Ok(Value::Unit))?
        }
        Expr::Variable(_, _) => todo!(),
        Expr::List(_) => todo!(),
        Expr::Seq(_) => todo!(),
        Expr::Table() => todo!(),
        Expr::Pattern() => todo!(),
        Expr::If { cond, yes, no } => todo!(),
        Expr::Match { expr, arms } => todo!(),
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
