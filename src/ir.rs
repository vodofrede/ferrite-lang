use crate::{Error, Expr};

/// Typed Intermediate Representation
#[derive(Debug)]
pub struct Tir {
    pub irs: Vec<Ir>,
}
pub fn lower(ast: Expr) -> Result<Tir, Error> {
    let mut irs = vec![];
    let tir = Tir { irs };
    Ok(tir)

    // for, while -> loop + match
    //
}
#[derive(Debug, Clone, Copy)]
pub enum Ir {
    Value,
    Label,
    Loop,
    Cond,
    Call,
    Return,
}
