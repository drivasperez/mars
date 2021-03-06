use super::metadata::{metadata, MetadataValue};
use super::numeric_expr::NumericExpr;
use super::{
    instruction::{comment, definition, instruction, org_statement, RawInstruction},
    numeric_expr::expr,
};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{line_ending, multispace0, space0},
    combinator::opt,
    combinator::{all_consuming, map},
    multi::separated_list,
    sequence::pair,
    sequence::preceded,
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Line<'a> {
    Instruction(RawInstruction<'a>),
    Comment(&'a str),
    OrgStatement(NumericExpr<'a>),
    MetadataStatement(MetadataValue<'a>),
    Definition {
        label: &'a str,
        definition: &'a str,
        full_definition: &'a str,
    },
}

fn line(i: &str) -> IResult<&str, Line> {
    terminated(
        delimited(
            space0,
            alt((
                map(definition, |(label, definition, full_definition)| {
                    Line::Definition {
                        label,
                        definition,
                        full_definition,
                    }
                }),
                map(org_statement, Line::OrgStatement),
                map(metadata, Line::MetadataStatement),
                map(comment, Line::Comment),
                map(instruction, Line::Instruction),
            )),
            space0,
        ),
        opt(comment),
    )(i)
}

pub(crate) fn lines(i: &str) -> IResult<&str, Vec<Line>> {
    all_consuming(preceded(
        multispace0,
        map(
            pair(
                separated_list(tuple((space0, line_ending, multispace0)), line),
                ending_line,
            ),
            |(mut list, end)| {
                if let Some(val) = end {
                    list.push(Line::OrgStatement(val));
                }
                list
            },
        ),
    ))(i)
}

fn ending_line(i: &str) -> IResult<&str, Option<NumericExpr>> {
    delimited(
        multispace0,
        preceded(pair(tag_no_case("END"), space0), opt(expr)),
        multispace0,
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::instruction::Operation;
    use crate::parser::instruction::*;
    use crate::parser::numeric_expr::*;
    #[test]
    fn test_dwarf() {
        let dwarf = include_str!("../../warriors/dwarf.red");
        lines(dwarf).unwrap();
    }

    #[test]
    fn test_imp() {
        let imp = include_str!("../../warriors/imp.red");
        lines(imp).unwrap();
    }

    #[test]
    fn one_label_with_colon() {
        line("imp:    mov.i   imp, imp+1").unwrap();
    }

    #[test]
    fn parse_lines() {
        let warrior = include_str!("../../warriors/dwarf.red");
        let (_, res) = lines(warrior).unwrap();

        assert_eq!(
            res,
            vec![
                Line::Comment(";redcode"),
                Line::MetadataStatement(MetadataValue::Name("Dwarf")),
                Line::MetadataStatement(MetadataValue::Author("A. K. Dewdney")),
                Line::MetadataStatement(MetadataValue::Version("94.1")),
                Line::MetadataStatement(MetadataValue::Date("April 29, 1993")),
                Line::MetadataStatement(MetadataValue::Strategy("Bombs every fourth instruction.")),
                Line::OrgStatement(NumericExpr::Value(ExprValue::Label("start"))),
                Line::Comment("; the label start should be the"),
                Line::Comment("; first to execute."),
                Line::Definition {
                    label: "step",
                    definition: "4                 ",
                    full_definition: "step    EQU      4                 "
                },
                Line::Comment("; with the character 4."),
                Line::Instruction(RawInstruction {
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
                Line::Instruction(RawInstruction {
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
                Line::Instruction(RawInstruction {
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
                Line::Instruction(RawInstruction {
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
