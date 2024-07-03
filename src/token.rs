use crate::Span;
use unicode_ident::{is_xid_continue, is_xid_start};

pub fn tokens(src: &str) -> Tokens {
    Tokens { src, index: 0 }
}
pub struct Tokens<'a> {
    src: &'a str,
    index: usize,
}
impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        const KEYWORDS: &[&str] = &["var", "do", "end", "if", "then", "else"];
        const OPERATORS: &str = "+-*/%^<>=.:!";
        const GROUPING: &str = "()[]";

        use Kind::*;
        let (kind, text) = match char_at(self.src, 0)? {
            '#' => (Comment, scan(self.src, |c| c != '\n')),
            '\n' => (Break, &self.src[..1]),
            c if c.is_whitespace() => (Space, scan(self.src, |c| c.is_whitespace() && c != '\n')),
            c if OPERATORS.contains(c) => (Operator, scan(self.src, |c| OPERATORS.contains(c))),
            c if GROUPING.contains(c) => (Grouping, scan(self.src, |c| GROUPING.contains(c))),
            c if c.is_ascii_digit() => {
                let mut found_dot = false;
                let text = scan(self.src, |c| {
                    let p = c.is_ascii_digit() || c == '_' || (c == '.' && !found_dot);
                    found_dot |= c == '.';
                    p
                });
                (Number, text)
            }
            '"' => (Text, scan(self.src, |c| c != '"')), // needs +1 char at the end
            c @ '_' | c if is_xid_start(c) => match scan(self.src, is_xid_continue) {
                text @ "true" | text @ "false" => (Bool, text),
                text @ "and" | text @ "or" | text @ "not" => (Operator, text),
                text if KEYWORDS.contains(&text) => (Keyword, text),
                text => (Identifier, text),
            },
            c => {
                eprintln!("syntax error: unhandled token: {c:?}");
                return None;
            }
        };

        let len = text.len();
        let span = Span {
            start: self.index,
            len,
        };
        self.index += len;
        self.src = &self.src[len..];

        match kind {
            Space | Break | Comment => self.next(),
            _ => Some(Token { text, kind, span }),
        }
    }
}
fn char_at(s: &str, i: usize) -> Option<char> {
    s.get(i..)?.chars().next()
}
fn scan(s: &str, mut valid: impl FnMut(char) -> bool) -> &str {
    let mut cursor = s.chars().peekable();
    let mut len = 0;
    while let Some(c) = cursor.peek().copied() {
        if !valid(c) {
            break;
        }
        cursor.next();
        len += 1
    }
    &s[..len]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: Kind,
    pub span: Span,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Space,
    Break,
    Comment,
    Operator,
    Grouping,
    Number,
    Text,
    Bool,
    Identifier,
    Keyword,
}
