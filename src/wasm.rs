use crate::{ir::Tir, Item, Error, Expr};

pub fn dump(ir: Tir) -> Vec<u8> {
    // organize sections

    dbg!(&ir);

    // imports
    // types
    // functions
    // export
    // code
    // memory
    // data
    // start

    // dump sections

    todo!()
}

// opcodes
const I32_CONST: u8 = 0x41;
const I32_EQ: u8 = 0x46;
const I32_NE: u8 = 0x47;
const I32_ADD: u8 = 0x6A;
const I32_SUB: u8 = 0x6B;
const I32_MUL: u8 = 0x6C;
const I32_DIV_S: u8 = 0x6D;
const I32_AND: u8 = 0x71;
const I32_OR: u8 = 0x72;
const I32_XOR: u8 = 0x73;

const F64_CONST: u8 = 0x44;
const F64_EQ: u8 = 0x61;
const F64_NE: u8 = 0x62;
const F64_LT: u8 = 0x63;
const F64_GT: u8 = 0x64;
const F64_LE: u8 = 0x65;
const F64_GE: u8 = 0x66;
const F64_ADD: u8 = 0xa0;
const F64_SUB: u8 = 0xa1;
const F64_MUL: u8 = 0xa2;
const F64_DIV: u8 = 0xa3;
