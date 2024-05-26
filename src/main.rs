#![warn(clippy::all)]

mod parse;
mod token;

use crate::{parse::*, token::*};
use std::{
    collections, env, error, fmt, fs,
    io::{prelude::*, stdin, stdout},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    match args.as_slice() {
        [_, file, ..] => {
            let src = fs::read_to_string(file)?;
            let ast = read(&src)?;
            let res = eval(ast, &mut Table::new())?;
            println!("{res:?}");
            Ok(())
        }
        _ => prompt(Table::new()),
    }
}
fn prompt(mut table: Table) -> Result<(), Box<dyn error::Error>> {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let (stdin, mut stdout) = (stdin(), stdout());
    let mut input = String::new();
    loop {
        print!("> ");
        stdout.flush()?;
        input.clear();
        stdin.read_line(&mut input)?;
        let Ok(ast) = read(&input) else { continue };
        let Ok(res) = eval(ast, &mut table) else {
            continue;
        };
        println!("{res:?}");
    }
}
fn read(src: &str) -> Result<Expr, Error> {
    parse(tokens(src))
}
fn eval(e: Expr, table: &mut Table) -> Result<Expr, Error> {
    use Expr::*;
    let e = match e {
        Unary(op, n) => {
            let n = eval(*n, table)?;
            match (op.as_str(), n) {
                ("-", Number(n)) => Number(-n),
                ("not", Bool(b)) => Bool(!b),
                (op, n) => return Err(error!("cannot apply unary {op} to {n:?}")),
            }
        }
        Binary(op, a, b) => match (op.as_str(), *a, *b) {
            ("=", Identifier(a), b) => {
                let b = eval(b, table)?;
                table.insert(a.to_string(), b);
                Unit
            }
            (op, a, b) => {
                let a = eval(a, table)?;
                let b = eval(b, table)?;
                match (op, a, b) {
                    ("+", Number(a), Number(b)) => Number(a + b),
                    ("-", Number(a), Number(b)) => Number(a - b),
                    ("*", Number(a), Number(b)) => Number(a * b),
                    ("/", Number(a), Number(b)) => Number(a / b),
                    ("%", Number(a), Number(b)) => Number(a % b),
                    ("^", Number(a), Number(b)) => Number(a.powf(b)),

                    ("==", Number(a), Number(b)) => Bool(a == b),
                    ("!=", Number(a), Number(b)) => Bool(a != b),
                    (">", Number(a), Number(b)) => Bool(a > b),
                    ("<", Number(a), Number(b)) => Bool(a < b),
                    (">=", Number(a), Number(b)) => Bool(a >= b),
                    ("<=", Number(a), Number(b)) => Bool(a <= b),

                    ("==", Bool(a), Bool(b)) => Bool(a == b),
                    ("!=", Bool(a), Bool(b)) => Bool(a != b),
                    ("and", Bool(a), Bool(b)) => Bool(a && b),
                    ("or", Bool(a), Bool(b)) => Bool(a || b),

                    (op, a, b) => return Err(error!("cannot apply {op} to {a:?} and {b:?}")),
                }
            }
        },
        Identifier(id) => table
            .get(&id)
            .ok_or(error!("undefined variable {id}"))?
            .clone(),
        Block(es) => {
            let mut local = table.clone();
            es.into_iter()
                .map(|e| eval(e, &mut local))
                .last()
                .unwrap_or(Ok(Unit))?
        }
        e => e,
    };
    Ok(e)
}
type Table = collections::HashMap<String, Expr>;

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
            write!(f, "{} at {}:{}", self.msg, span.line, span.column)
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
