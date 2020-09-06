use crate::parser::numeric_expr::NumericExpr;

#[derive(Debug, PartialEq, Eq)]
pub enum Opcode {
    Dat,
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Jmp,
    Jmz,
    Jmn,
    Djn,
    Slt,
    Seq,
    Sne,
    Spl,
    Nop,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Term<'a> {
    Label(&'a str),
    Number(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumericOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionListItem<'a> {
    TermItem(Term<'a>),
    Operation(NumericOperation),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Instruction<'a> {
    pub label_list: Vec<&'a str>,
    pub operation: Operation,
    pub field_a: Address<'a>,
    pub field_b: Option<Address<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Address<'a> {
    pub expr: NumericExpr<'a>,
    pub mode: AddressMode,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Operation {
    pub opcode: Opcode,
    pub modifier: Modifier,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddressMode {
    Direct,
    Immediate,
    Indirect,
    PredecrementIndirect,
    PostincrementIndirect,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Modifier {
    A,
    B,
    AB,
    BA,
    F,
    X,
    I,
}
