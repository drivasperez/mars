use crate::error::ParseError;
use crate::types::*;
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_till},
    character::complete::{
        alpha1, alphanumeric0, char, multispace1, not_line_ending, one_of, space0, space1,
    },
    combinator::{map, opt, peek, recognize},
    multi::{many0, separated_list},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

pub mod numeric_expr;

use numeric_expr::{expr, NumericExpr};

pub fn parse(i: &str) -> Result<Vec<Line>, ParseError> {
    let (_, ls) = lines(i).map_err(|e| match e {
        nom::Err::Incomplete(_) => ParseError::Incomplete,
        nom::Err::Error((_, ek)) | nom::Err::Failure((_, ek)) => ParseError::Parse(ek),
    })?;

    Ok(ls)
}

pub fn replace_definitions<'a>(s: &'a str) -> Result<String, Box<dyn std::error::Error + 'a>> {
    let (_, ls) = lines(s)?;

    let mut replaced = String::from(s);

    for line in ls {
        if let Line::Definition(label, def) = line {
            replaced = replaced.replace(label, def.trim());
        }
    }

    Ok(replaced)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Line<'a> {
    Instruction(Instruction<'a>),
    Comment(&'a str),
    OrgStatement(NumericExpr<'a>),
    Definition(&'a str, &'a str),
}

fn line(i: &str) -> IResult<&str, Line> {
    let (i, _) = space0(i)?;

    let (i, l) = if peek(definition)(i).is_ok() {
        let (i, (s, e)) = definition(i)?;
        (i, Line::Definition(s, e))
    } else if peek(comment)(i).is_ok() {
        let (i, l) = comment(i)?;
        (i, Line::Comment(l))
    } else if peek(org_statement)(i).is_ok() {
        let (i, l) = org_statement(i)?;
        (i, Line::OrgStatement(l))
    } else {
        let (i, l) = instruction(i)?;
        (i, Line::Instruction(l))
    };

    let (i, _) = space0(i)?;

    Ok((i, l))
}

pub fn lines(i: &str) -> IResult<&str, Vec<Line>> {
    let (i, ls) = separated_list(multispace1, line)(i)?;
    let (i, _) = opt(tag_no_case("END"))(i)?;
    Ok((i, ls))
}

fn definition(i: &str) -> IResult<&str, (&str, &str)> {
    let (i, label) = label(i)?;
    let (i, _) = recognize(tuple((space1, tag_no_case("EQU"), space1)))(i)?;
    let (i, expression) = take_till(|c| c == ';' || c == '\n' || c == '\r')(i)?;
    let (i, _) = opt(comment)(i)?;

    Ok((i, (label, expression)))
}

fn org_statement(i: &str) -> IResult<&str, NumericExpr> {
    let (i, _) = recognize(tuple((space0, tag_no_case("ORG"), space1)))(i)?;
    let (i, res) = expr(i)?;
    let (i, _) = opt(preceded(space0, comment))(i)?;

    Ok((i, res))
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

fn operation(i: &str) -> IResult<&str, Operation> {
    map(
        pair(opcode, opt(preceded(char('.'), modifier))),
        |(opcode, modifier)| Operation {
            modifier: modifier.unwrap_or_else(|| opcode.default_modifier()),
            opcode,
        },
    )(i)
}

fn address(i: &str) -> IResult<&str, Address> {
    let (i, (mode, expression)) = pair(opt(address_mode), expr)(i)?;
    let mode = mode.unwrap_or(AddressMode::Direct);

    Ok((
        i,
        Address {
            mode,
            expr: expression,
        },
    ))
}

fn instruction(i: &str) -> IResult<&str, Instruction> {
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

fn label(i: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, alphanumeric0))(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    recognize(tuple((char(';'), not_line_ending)))(i)
}

fn label_list(i: &str) -> IResult<&str, Vec<&str>> {
    terminated(many0(terminated(label, multispace1)), opt(char(':')))(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use numeric_expr::ExprValue;

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
        assert_eq!(res, ("step", "4"));

        let (i, res) = definition(
            "step   EQU blah + 4 / 2 * something ; here is a comment about this definition",
        )
        .unwrap();
        assert_eq!(i, "");
        assert_eq!(res, ("step", "blah + 4 / 2 * something "));

        let (i, res) = definition(
            "b33    EQU      4                 ; Replaces all occurrences of 'step'
",
        )
        .unwrap();
        assert_eq!(res, ("b33", "4                 "));
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

    #[test]
    fn test_replace_definitions() {
        let warrior = include_str!("../../warriors/dwarf.red");
        let replaced = replace_definitions(warrior).unwrap();

        assert_eq!(replaced, warrior.replace("step", "4"));
        assert!(lines(&replaced).is_ok());
    }

    #[test]
    fn test_warriors() {
        let dwarf = include_str!("../../warriors/dwarf.red");
        let imp = include_str!("../../warriors/imp.red");

        assert!(lines(dwarf).is_ok());
        assert!(lines(imp).is_ok());
    }

    #[test]
    fn parse_lines() {
        let warrior = include_str!("../../warriors/dwarf.red");
        let (_, res) = lines(warrior).unwrap();

        assert_eq!(
            res,
            vec![
                Line::Comment(";redcode"),
                Line::Comment(";name          Dwarf"),
                Line::Comment(";author        A. K. Dewdney"),
                Line::Comment(";version       94.1"),
                Line::Comment(";date          April 29, 1993"),
                Line::Comment(";strategy      Bombs every fourth instruction."),
                Line::OrgStatement(NumericExpr::Value(ExprValue::Label("start"))),
                Line::Comment("; the label start should be the"),
                Line::Comment("; first to execute."),
                Line::Definition("step", "4                 "),
                Line::Comment("; with the character 4."),
                Line::Instruction(Instruction {
                    label_list: vec!["target"],
                    operation: Operation {
                        opcode: Opcode::Dat,
                        modifier: Modifier::F
                    },
                    field_a: Address {
                        expr: NumericExpr::Value(ExprValue::Number(0)),
                        mode: AddressMode::Immediate
                    },
                    field_b: Some(Address {
                        expr: NumericExpr::Value(ExprValue::Number(0)),
                        mode: AddressMode::Immediate
                    })
                }),
                Line::Instruction(Instruction {
                    label_list: vec!["start"],
                    operation: Operation {
                        opcode: Opcode::Add,
                        modifier: Modifier::AB
                    },
                    field_a: Address {
                        expr: NumericExpr::Value(ExprValue::Label("step")),
                        mode: AddressMode::Immediate
                    },
                    field_b: Some(Address {
                        expr: NumericExpr::Value(ExprValue::Label("target")),
                        mode: AddressMode::Direct
                    })
                }),
                Line::Instruction(Instruction {
                    label_list: vec![],
                    operation: Operation {
                        opcode: Opcode::Mov,
                        modifier: Modifier::AB
                    },
                    field_a: Address {
                        expr: NumericExpr::Value(ExprValue::Number(0)),
                        mode: AddressMode::Immediate
                    },
                    field_b: Some(Address {
                        expr: NumericExpr::Value(ExprValue::Label("target")),
                        mode: AddressMode::BFieldIndirect
                    })
                }),
                Line::Instruction(Instruction {
                    label_list: vec![],
                    operation: Operation {
                        opcode: Opcode::Jmp,
                        modifier: Modifier::A
                    },
                    field_a: Address {
                        expr: NumericExpr::Value(ExprValue::Label("start")),
                        mode: AddressMode::Direct
                    },
                    field_b: None
                }),
                Line::Comment("; the instruction labelled start.")
            ]
        )
    }
}
