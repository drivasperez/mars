use crate::parser::numeric_expr::{ExprValue, NumericExpr};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

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

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Opcode::*;
        write!(
            f,
            "{}",
            match self {
                Dat => "DAT",
                Mov => "MOV",
                Add => "ADD",
                Sub => "SUB",
                Mul => "MUL",
                Div => "DIV",
                Mod => "MOD",
                Jmp => "JMP",
                Jmz => "JMZ",
                Jmn => "JMN",
                Djn => "DJN",
                Slt => "SLT",
                Seq => "SEQ",
                Sne => "SNE",
                Spl => "SPL",
                Nop => "NOP",
            }
        )
    }
}

impl Opcode {
    pub fn default_modifier(&self) -> Modifier {
        match self {
            Opcode::Dat | Opcode::Nop => Modifier::F,
            Opcode::Mov | Opcode::Seq | Opcode::Sne => Modifier::I,
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod => Modifier::AB,
            Opcode::Jmp | Opcode::Jmz | Opcode::Jmn | Opcode::Djn | Opcode::Slt | Opcode::Spl => {
                Modifier::B
            }
        }
    }
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

impl std::default::Default for Address<'_> {
    fn default() -> Self {
        Self {
            mode: AddressMode::Direct,
            expr: NumericExpr::Value(ExprValue::Number(0)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Operation {
    pub opcode: Opcode,
    pub modifier: Modifier,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddressMode {
    Immediate,
    Direct,
    AFieldIndirect,
    BFieldIndirect,
    AFieldPredecrementIndirect,
    BFieldPredecrementIndirect,
    AFieldPostincrementIndirect,
    BFieldPostincrementIndirect,
}

impl Display for AddressMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use AddressMode::*;
        write!(
            f,
            "{}",
            match self {
                Immediate => "#",
                Direct => "$",
                AFieldIndirect => "*",
                BFieldIndirect => "@",
                AFieldPredecrementIndirect => "{",
                BFieldPredecrementIndirect => "<",
                AFieldPostincrementIndirect => "}",
                BFieldPostincrementIndirect => ">",
            }
        )
    }
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

impl Display for Modifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Modifier::*;
        write!(
            f,
            "{}",
            match self {
                A => "A",
                B => "B",
                AB => "AB",
                BA => "BA",
                F => "F",
                X => "X",
                I => "I",
            }
        )
    }
}

pub struct RawInstruction {
    opcode: Opcode,
    modifier: Modifier,
    addr1: (AddressMode, i32),
    addr2: (AddressMode, i32),
}

impl RawInstruction {
    pub fn new(
        opcode: Opcode,
        modifier: Modifier,
        addr1: (AddressMode, i32),
        addr2: (AddressMode, i32),
    ) -> Self {
        Self {
            opcode,
            modifier,
            addr1,
            addr2,
        }
    }
}

impl RawInstruction {
    pub fn from_instruction(
        instruction: Instruction,
        labels: &HashMap<&str, i32>,
        current_line: usize,
    ) -> Result<RawInstruction, EvaluateError> {
        let Instruction {
            label_list: _,
            operation,
            field_a,
            field_b,
        } = instruction;

        let Address { mode, expr } = field_a;
        let addr1 = (mode, expr.evaluate_relative(labels, current_line as i32)?);
        let Address { mode, expr } = field_b.unwrap_or_default();
        let addr2 = (mode, expr.evaluate_relative(labels, current_line as i32)?);

        let Operation { opcode, modifier } = operation;

        Ok(RawInstruction::new(opcode, modifier, addr1, addr2))
    }
}

impl Display for RawInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} {}{}, {}{}",
            self.opcode, self.modifier, self.addr1.0, self.addr1.1, self.addr2.0, self.addr2.1
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn display_raw_instruction() {
        let inst = RawInstruction {
            opcode: Opcode::Mov,
            modifier: Modifier::BA,
            addr1: (AddressMode::Direct, 8),
            addr2: (AddressMode::AFieldIndirect, 2),
        };

        assert_eq!(format!("{}", inst), String::from("MOV.BA $8, *2"));
    }
}
