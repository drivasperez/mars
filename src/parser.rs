use crate::types::*;
use nom::branch;
use nom::bytes::complete as bytes;
use nom::character::complete as character;
use nom::IResult;

fn line(i: &str) -> IResult<&str, Line> {
    use nom::combinator::peek;
    let (i, _) = character::space0(i)?;

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

    let (i, _) = character::space0(i)?;

    Ok((i, l))
}

fn lines(i: &str) -> IResult<&str, Vec<Line>> {
    let (i, ls) = nom::multi::separated_list(character::multispace1, line)(i)?;
    let (i, _) = nom::combinator::opt(bytes::tag_no_case("END"))(i)?;
    Ok((i, ls))
}

fn definition(i: &str) -> IResult<&str, (&str, Expression)> {
    let (i, label) = label(i)?;
    let (i, _) = nom::combinator::recognize(nom::sequence::tuple((
        character::space1,
        bytes::tag_no_case("EQU"),
        character::space1,
    )))(i)?;
    let (i, expression) = expr(i)?;
    let (i, _) = nom::combinator::opt(nom::sequence::preceded(character::space0, comment))(i)?;

    Ok((i, (label, expression)))
}

fn org_statement(i: &str) -> IResult<&str, Expression> {
    let (i, _) = nom::combinator::recognize(nom::sequence::tuple((
        character::space0,
        bytes::tag_no_case("ORG"),
        character::space1,
    )))(i)?;
    let (i, res) = expr(i)?;
    let (i, _) = nom::combinator::opt(nom::sequence::preceded(character::space0, comment))(i)?;

    Ok((i, res))
}

fn address_mode(i: &str) -> IResult<&str, AddressMode> {
    let (i, symbol) = character::one_of("#$@<>")(i)?;

    let mode = match symbol {
        '$' => AddressMode::Direct,
        '#' => AddressMode::Immediate,
        '@' => AddressMode::Indirect,
        '<' => AddressMode::PredecrementIndirect,
        '>' => AddressMode::PostincrementIndirect,
        _ => unreachable!(),
    };

    Ok((i, mode))
}

fn opcode(i: &str) -> IResult<&str, Opcode> {
    use bytes::tag_no_case as t;
    let (i, opcode) = branch::alt((
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
    ))(i)?;

    let opcode = match opcode.to_ascii_uppercase().as_str() {
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
    };

    Ok((i, opcode))
}

fn modifier(i: &str) -> IResult<&str, Modifier> {
    use bytes::tag_no_case as t;
    let (i, modifier) = branch::alt((t("AB"), t("BA"), t("A"), t("B"), t("F"), t("X"), t("I")))(i)?;

    let modifier = match modifier.to_ascii_uppercase().as_str() {
        "A" => Modifier::A,
        "B" => Modifier::B,
        "AB" => Modifier::AB,
        "BA" => Modifier::BA,
        "F" => Modifier::F,
        "X" => Modifier::X,
        "I" => Modifier::I,
        _ => unreachable!(),
    };

    Ok((i, modifier))
}

fn operation(i: &str) -> IResult<&str, Operation> {
    let (i, opcode) = opcode(i)?;
    let (i, modifier) =
        nom::combinator::opt(nom::sequence::preceded(character::char('.'), modifier))(i)?;

    let modifier = modifier.unwrap_or(match opcode {
        Opcode::Dat | Opcode::Nop => Modifier::F,
        Opcode::Mov | Opcode::Seq | Opcode::Sne => Modifier::I,
        Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod => Modifier::AB,
        Opcode::Jmp | Opcode::Jmz | Opcode::Jmn | Opcode::Djn | Opcode::Slt | Opcode::Spl => {
            Modifier::B
        }
    });

    let operation = Operation { opcode, modifier };

    Ok((i, operation))
}

fn address(i: &str) -> IResult<&str, Address> {
    let (i, (mode, expression)) = nom::sequence::pair(nom::combinator::opt(address_mode), expr)(i)?;
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
    let (i, _) = character::space0(i)?;
    let (i, labels) = label_list(i)?;
    let (i, op) = operation(i)?;
    let (i, _) = character::space1(i)?;
    let (i, addr1) = address(i)?;
    let (i, _) = character::space0(i)?;
    let (i, addr2) = nom::combinator::opt(nom::sequence::preceded(
        nom::sequence::tuple((character::space0, character::char(','), character::space0)),
        address,
    ))(i)?;

    let (i, _) = character::space0(i)?;

    let (i, _) = nom::combinator::opt(comment)(i)?;

    let instruction = Instruction {
        label_list: labels,
        operation: op,
        field_a: addr1,
        field_b: addr2,
    };

    Ok((i, instruction))
}

fn number(i: &str) -> IResult<&str, i32> {
    let (i, (sign, num)) = nom::sequence::pair(
        nom::combinator::opt(character::one_of("+-")),
        character::digit1,
    )(i)?;

    let sign = sign.unwrap_or('+');
    let mut num: i32 = num.parse().unwrap();

    if sign == '-' {
        num *= -1;
    }

    Ok((i, num))
}

fn label(i: &str) -> IResult<&str, &str> {
    let (i, label) = nom::combinator::recognize(nom::sequence::pair(
        character::alpha1,
        character::alphanumeric0,
    ))(i)?;

    Ok((i, label))
}

fn term(i: &str) -> IResult<&str, Term> {
    let (i, term) = if let Ok(_) = nom::combinator::peek(number)(i) {
        let (i, term) = number(i)?;
        let term = Term::Number(term);
        (i, term)
    } else {
        let (i, term) = label(i)?;
        let term = Term::Label(term);
        (i, term)
    };

    Ok((i, term))
}

fn expr(i: &str) -> IResult<&str, Expression> {
    let (i, (head, tail)) = nom::sequence::pair(
        term,
        nom::multi::many0(nom::sequence::pair(
            nom::sequence::preceded(character::space0, character::one_of("+-*/%")),
            nom::sequence::preceded(character::space0, term),
        )),
    )(i)?;

    let head = ExpressionListItem::TermItem(head);
    let mut tail: Vec<ExpressionListItem> = tail
        .into_iter()
        .flat_map(|(op, term)| {
            let op = match op {
                '+' => NumericOperation::Add,
                '-' => NumericOperation::Subtract,
                '*' => NumericOperation::Multiply,
                '/' => NumericOperation::Divide,
                '%' => NumericOperation::Modulo,
                _ => unreachable!(),
            };

            vec![
                ExpressionListItem::Operation(op),
                ExpressionListItem::TermItem(term),
            ]
        })
        .collect();

    let mut v = Vec::new();
    v.push(head);
    v.append(&mut tail);

    Ok((i, v))
}

fn comment(i: &str) -> IResult<&str, &str> {
    let (i, res) = nom::combinator::recognize(nom::sequence::tuple((
        character::char(';'),
        character::not_line_ending,
    )))(i)?;

    Ok((i, res))
}

fn label_list(i: &str) -> IResult<&str, Vec<&str>> {
    let res = nom::multi::many0(nom::sequence::terminated(label, character::multispace1))(i)?;

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

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
    fn parse_number() {
        let res = number("322");
        assert_eq!(res, Ok(("", 322)));

        let res = number("+9");
        assert_eq!(res, Ok(("", 9)));

        let res = number("-545");
        assert_eq!(res, Ok(("", -545)));

        let res = number("-545abc");
        assert_eq!(res, Ok(("abc", -545)));

        let res = number("323.32");
        assert_eq!(res, Ok((".32", 323)));

        let res = number("u323");
        assert!(res.is_err());
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
    fn parse_expression() {
        let (i, res) = expr("3 - 5 + hmm / 2 * 22").unwrap();

        assert_eq!(i, "");
        assert_eq!(res.len(), 9);
    }

    #[test]
    fn parse_definition() {
        let (i, res) = definition("step   EQU 4").unwrap();
        assert_eq!(i, "");
        assert_eq!(
            res,
            ("step", vec![ExpressionListItem::TermItem(Term::Number(4))])
        );

        let (i, res) = definition("step   EQU blah + 4 / 2 * something").unwrap();
        assert_eq!(i, "");
        assert_eq!(
            res,
            (
                "step",
                vec![
                    ExpressionListItem::TermItem(Term::Label("blah")),
                    ExpressionListItem::Operation(NumericOperation::Add),
                    ExpressionListItem::TermItem(Term::Number(4)),
                    ExpressionListItem::Operation(NumericOperation::Divide),
                    ExpressionListItem::TermItem(Term::Number(2)),
                    ExpressionListItem::Operation(NumericOperation::Multiply),
                    ExpressionListItem::TermItem(Term::Label("something"))
                ]
            )
        );
    }

    #[test]
    fn parse_org_statement() {
        let (i, res) = org_statement("ORG 3 + ser").unwrap();
        assert_eq!(i, "");
        assert_eq!(
            res,
            vec![
                ExpressionListItem::TermItem(Term::Number(3)),
                ExpressionListItem::Operation(NumericOperation::Add),
                ExpressionListItem::TermItem(Term::Label("ser"))
            ]
        );

        let (i, res) = org_statement("    ORG   flip").unwrap();
        assert_eq!(i, "");
        assert_eq!(res, vec![ExpressionListItem::TermItem(Term::Label("flip"))]);
    }

    #[test]
    fn parse_lines() {
        let (i, res) = lines(
            ";redcode
     
 ;name          Dwarf
 ;author        A. K. Dewdney
 ;version       94.1
 ;date          April 29, 1993
 
 ;strategy      Bombs every fourth instruction.
 
         ORG     start              ; Indicates the instruction with
                                    ; the label 'start' should be the
                                    ; first to execute.
 
 step    EQU      4                 ; Replaces all occurrences of 'step'
                                    ; with the character '4'.
 
 target  DAT.F   #0,     #0         ; Pointer to target instruction.
 start   ADD.AB  #step,   target    ; Increments pointer by step.
         MOV.AB  #0,     @target    ; Bombs target instruction.
         JMP.A    start             ; Same as JMP.A -2.  Loops back to
                                    ; the instruction labelled 'start'.
         END",
        )
        .unwrap();

        assert_eq!(
            res,
            vec![
                Line::Comment(";redcode"),
                Line::Comment(";name          Dwarf"),
                Line::Comment(";author        A. K. Dewdney"),
                Line::Comment(";version       94.1"),
                Line::Comment(";date          April 29, 1993"),
                Line::Comment(";strategy      Bombs every fourth instruction."),
                Line::OrgStatement(vec![ExpressionListItem::TermItem(Term::Label("start"))]),
                Line::Comment("; the label \'start\' should be the"),
                Line::Comment("; first to execute."),
                Line::Definition("step", vec![ExpressionListItem::TermItem(Term::Number(4))]),
                Line::Comment("; with the character \'4\'."),
                Line::Instruction(Instruction {
                    label_list: vec!["target"],
                    operation: Operation {
                        opcode: Opcode::Dat,
                        modifier: Modifier::F
                    },
                    field_a: Address {
                        expr: vec![ExpressionListItem::TermItem(Term::Number(0))],
                        mode: AddressMode::Immediate
                    },
                    field_b: Some(Address {
                        expr: vec![ExpressionListItem::TermItem(Term::Number(0))],
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
                        expr: vec![ExpressionListItem::TermItem(Term::Label("step"))],
                        mode: AddressMode::Immediate
                    },
                    field_b: Some(Address {
                        expr: vec![ExpressionListItem::TermItem(Term::Label("target"))],
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
                        expr: vec![ExpressionListItem::TermItem(Term::Number(0))],
                        mode: AddressMode::Immediate
                    },
                    field_b: Some(Address {
                        expr: vec![ExpressionListItem::TermItem(Term::Label("target"))],
                        mode: AddressMode::Indirect
                    })
                }),
                Line::Instruction(Instruction {
                    label_list: vec![],
                    operation: Operation {
                        opcode: Opcode::Jmp,
                        modifier: Modifier::A
                    },
                    field_a: Address {
                        expr: vec![ExpressionListItem::TermItem(Term::Label("start"))],
                        mode: AddressMode::Direct
                    },
                    field_b: None
                }),
                Line::Comment("; the instruction labelled \'start\'.")
            ]
        )
    }
}
