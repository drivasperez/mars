use crate::types::Modifier;
use nom::{branch::alt, bytes::complete::tag_no_case, combinator::map, IResult};
use std::fmt::{Display, Formatter};

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

fn opcode(i: &str) -> IResult<&str, Opcode> {
    use tag_no_case as t;
    map(
        alt((
            t("DAT"),
            t("MOV"),
            t("ADD"),
            t("SUB"),
            t("MUL"),
            t("DIV"),
            t("MOD"),
            t("JMP"),
            t("JMZ"),
            t("JMN"),
            t("DJN"),
            t("CMP"),
            t("SLT"),
            t("SPL"),
            t("SEQ"),
            t("SNE"),
            t("ORG"),
            t("EQU"),
            t("NOP"),
        )),
        |opcode: &str| match opcode.to_ascii_uppercase().as_str() {
            "DAT" => Opcode::Dat,
            "MOV" => Opcode::Mov,
            "ADD" => Opcode::Add,
            "SUB" => Opcode::Sub,
            "MUL" => Opcode::Mul,
            "DIV" => Opcode::Div,
            "MOD" => Opcode::Mod,
            "JMP" => Opcode::Jmp,
            "JMZ" => Opcode::Jmz,
            "JMN" => Opcode::Jmn,
            "DJN" => Opcode::Djn,
            "CMP" => Opcode::Seq,
            "SLT" => Opcode::Slt,
            "SPL" => Opcode::Spl,
            "SEQ" => Opcode::Seq,
            "SNE" => Opcode::Sne,
            "NOP" => Opcode::Nop,
            _ => unreachable!(),
        },
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_opcode() {
        let res = opcode("DAT#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Dat)));

        let res = opcode("MOV#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Mov)));

        let res = opcode("Add#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Add)));

        let res = opcode("SUB#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Sub)));

        let res = opcode("MUL#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Mul)));

        let res = opcode("div#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Div)));

        let res = opcode("MOD#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Mod)));

        let res = opcode("JMP#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Jmp)));

        let res = opcode("JMZ#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Jmz)));

        let res = opcode("jmn#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Jmn)));

        let res = opcode("DJN#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Djn)));

        let res = opcode("CMP#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Seq)));

        let res = opcode("sLT#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Slt)));

        let res = opcode("SPL#2323");
        assert_eq!(res, Ok(("#2323", Opcode::Spl)));

        let res = opcode("2321DAT23");
        assert!(res.is_err());
    }
}
