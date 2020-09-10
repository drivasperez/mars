use super::numeric_expr::{expr, ExprValue, NumericExpr};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_till},
    character::complete::{
        alpha1, alphanumeric0, char, multispace1, not_line_ending, one_of, space0, space1,
    },
    combinator::not,
    combinator::{map, opt, peek, recognize},
    multi::many0,
    multi::separated_list,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use std::fmt::{Display, Formatter};

// Structs and Enums

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Instruction<'a> {
    pub label_list: Vec<&'a str>,
    pub operation: Operation,
    pub field_a: Address<'a>,
    pub field_b: Option<Address<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Address<'a> {
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

// Parsers

fn operation(i: &str) -> IResult<&str, Operation> {
    map(
        pair(opcode, opt(preceded(char('.'), modifier))),
        |(opcode, modifier)| Operation {
            modifier: modifier.unwrap_or_else(|| opcode.default_modifier()),
            opcode,
        },
    )(i)
}

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

fn address_mode(i: &str) -> IResult<&str, AddressMode> {
    use AddressMode::*;
    map(one_of("#$@*{<}>"), |symbol| match symbol {
        '#' => Immediate,
        '$' => Direct,
        '*' => AddressMode::AFieldIndirect,
        '@' => BFieldIndirect,
        '{' => AFieldPredecrementIndirect,
        '<' => BFieldPredecrementIndirect,
        '}' => AFieldPostincrementIndirect,
        '>' => BFieldPostincrementIndirect,
        _ => unreachable!(),
    })(i)
}

fn modifier(i: &str) -> IResult<&str, Modifier> {
    use tag_no_case as t;
    map(
        alt((t("AB"), t("BA"), t("A"), t("B"), t("F"), t("X"), t("I"))),
        |modifier: &str| match modifier.to_ascii_uppercase().as_str() {
            "A" => Modifier::A,
            "B" => Modifier::B,
            "AB" => Modifier::AB,
            "BA" => Modifier::BA,
            "F" => Modifier::F,
            "X" => Modifier::X,
            "I" => Modifier::I,
            _ => unreachable!(),
        },
    )(i)
}

fn address(i: &str) -> IResult<&str, Address> {
    map(pair(opt(address_mode), expr), |(mode, expr)| Address {
        mode: mode.unwrap_or(AddressMode::Direct),
        expr,
    })(i)
}

pub(super) fn instruction(i: &str) -> IResult<&str, Instruction> {
    let (i, _) = space0(i)?;
    let (i, labels) = label_list(i)?;
    let (i, op) = operation(i)?;
    let (i, _) = space1(i)?;
    let (i, addr1) = address(i)?;
    let (i, _) = space0(i)?;
    let (i, addr2) = opt(preceded(tuple((space0, char(','), space0)), address))(i)?;

    let (i, _) = space0(i)?;

    let (i, _) = opt(comment)(i)?;

    let instruction = Instruction {
        label_list: labels,
        operation: op,
        field_a: addr1,
        field_b: addr2,
    };

    Ok((i, instruction))
}

pub(crate) fn label(i: &str) -> IResult<&str, &str> {
    peek(not(opcode))(i)?;
    recognize(pair(alpha1, alphanumeric0))(i)
}

pub(super) fn comment(i: &str) -> IResult<&str, &str> {
    recognize(tuple((char(';'), not_line_ending)))(i)
}

fn label_list(i: &str) -> IResult<&str, Vec<&str>> {
    terminated(
        separated_list(multispace1, label),
        delimited(space0, opt(char(':')), space0),
    )(i)
}

pub(super) fn definition(i: &str) -> IResult<&str, (&str, &str, &str)> {
    let (i, full_definition) = peek(recognize(tuple((
        label,
        space1,
        tag_no_case("EQU"),
        space1,
        take_till(|c| c == ';' || c == '\n' || c == '\r'),
    ))))(i)?;
    let (i, label) = label(i)?;
    let (i, _) = recognize(tuple((space1, tag_no_case("EQU"), space1)))(i)?;
    let (i, expression) = take_till(|c| c == ';' || c == '\n' || c == '\r')(i)?;
    let (i, _) = opt(comment)(i)?;

    Ok((i, (label, expression, full_definition)))
}

pub(super) fn org_statement(i: &str) -> IResult<&str, NumericExpr> {
    delimited(
        recognize(tuple((space0, tag_no_case("ORG"), space1))),
        expr,
        opt(preceded(space0, comment)),
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

    #[test]
    fn parse_address_mode() {
        if let Ok((i, symbol)) = address_mode("#.hello") {
            assert_eq!(i, ".hello");
            assert_eq!(symbol, AddressMode::Immediate);
        } else {
            panic!();
        }
    }

    #[test]
    fn parse_modifier() {
        let res = modifier("A");
        assert_eq!(res, Ok(("", Modifier::A)));

        let res = modifier("323");
        assert!(res.is_err());

        let res = modifier("Ab#3aa");
        assert_eq!(res, Ok(("#3aa", Modifier::AB)));
    }

    #[test]
    fn parse_operation() {
        let (rest, op) = operation("MOV.AB").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Mov,
                modifier: Modifier::AB
            }
        );
        let (rest, op) = operation("mov.a").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Mov,
                modifier: Modifier::A
            }
        );
        let (rest, op) = operation("nop.BA").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Nop,
                modifier: Modifier::BA
            }
        );
        let (rest, op) = operation("ADD").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Add,
                modifier: Modifier::AB
            }
        );

        let (rest, op) = operation("JMZ").unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Jmz,
                modifier: Modifier::B
            }
        );

        let res = operation("boaelhaoente");
        assert!(res.is_err());

        let (rest, op) = operation("MOV#AB").unwrap();
        assert_eq!(rest, "#AB");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Mov,
                modifier: Modifier::I
            }
        );

        let (rest, op) = operation("SPL.4343").unwrap();
        assert_eq!(rest, ".4343");
        assert_eq!(
            op,
            Operation {
                opcode: Opcode::Spl,
                modifier: Modifier::B
            }
        );
    }

    #[test]
    fn parse_label() {
        let (i, l) = label("blah323blah").unwrap();
        assert_eq!(i, "");
        assert_eq!(l, "blah323blah");

        let (i, l) = label("blah 323blah").unwrap();
        assert_eq!(i, " 323blah");
        assert_eq!(l, "blah");

        assert!(label("0hello").is_err());
        assert!(label(";hello").is_err());
        assert!(label("'hello").is_err());

        label("MOV").unwrap_err();
        label("    DAT").unwrap_err();
    }

    #[test]
    fn parse_label_list() {
        let (i, l) = label_list("hello goodbye what3ver ").unwrap();
        assert_eq!(i, "");
        assert_eq!(l, vec!["hello", "goodbye", "what3ver"]);

        let (i, l) = label_list(
            "hello
            goodbye 
            what3ver ",
        )
        .unwrap();
        assert_eq!(i, "");
        assert_eq!(l, vec!["hello", "goodbye", "what3ver"]);

        let (i, l) = label_list("32uhoh these will parse").unwrap();
        assert_eq!(i, "32uhoh these will parse");
        assert_eq!(l, Vec::<&str>::new());

        let (i, l) = label_list("").unwrap();
        assert_eq!(i, "");
        assert_eq!(l, Vec::<&str>::new());

        let (i, l) = label_list("imp: mov.i").unwrap();
        assert_eq!(i, "mov.i");
    }

    #[test]
    fn parse_instruction() {
        let (i, _) =
            instruction("target  DAT.F   #0,     #0         ; Pointer to target instruction.\n")
                .unwrap();

        assert_eq!(i, "\n");

        let (i, instr) = instruction(
            "start
                something else
                ADD.AB  #step,   target    ; Increments pointer by step.",
        )
        .unwrap();
        assert_eq!(instr.label_list, vec!["start", "something", "else"]);

        assert_eq!(i, "");

        let (i, _) = instruction("MOV.AB  #0,     @target    ; Bombs target instruction.").unwrap();

        assert_eq!(i, "");

        let (i, instr) =
            instruction("         JMP.A    start             ; Same as JMP.A -2.  Loops back to")
                .unwrap();

        assert_eq!(i, "");
        assert_eq!(instr.label_list.len(), 0);
    }

    #[test]
    fn parse_definition() {
        let (i, res) = definition("step   EQU 4").unwrap();
        assert_eq!(i, "");
        assert_eq!(res, ("step", "4", "step   EQU 4"));

        let (i, res) = definition(
            "step   EQU blah + 4 / 2 * something ; here is a comment about this definition",
        )
        .unwrap();
        assert_eq!(i, "");
        assert_eq!(
            res,
            (
                "step",
                "blah + 4 / 2 * something ",
                "step   EQU blah + 4 / 2 * something "
            )
        );

        let (i, res) = definition(
            "b33    EQU      4                 ; Replaces all occurrences of 'step'
",
        )
        .unwrap();
        assert_eq!(
            res,
            (
                "b33",
                "4                 ",
                "b33    EQU      4                 "
            )
        );
        assert_eq!(i, "\n")
    }

    #[test]
    fn parse_org_statement() {
        let (i, res) = org_statement("ORG 3 + ser").unwrap();
        assert_eq!(i, "");
        assert_eq!(format!("{}", res), String::from("3 + ser"),);

        let (i, res) = org_statement("    ORG   flip").unwrap();
        assert_eq!(i, "");
        assert_eq!(format!("{}", res), String::from("flip"));
    }
}
