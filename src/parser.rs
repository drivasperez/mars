use nom::branch;
use nom::bytes::complete as bytes;
use nom::character::complete as character;
use nom::IResult;

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

pub enum Term<'a> {
    Label(&'a str),
    Number(i32),
}
pub enum NumericOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

enum ExpressionListItem<'a> {
    TermItem(Term<'a>),
    Operation(NumericOperation),
}

pub struct Instruction<'a> {
    label_list: Vec<&'a str>,
    operation: Operation,
    field_a: Address<'a>,
    field_b: Option<Address<'a>>,
}

pub struct Address<'a> {
    expr: Vec<ExpressionListItem<'a>>,
    mode: AddressMode,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Operation {
    opcode: Opcode,
    modifier: Modifier,
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

fn expr(i: &str) -> IResult<&str, Vec<ExpressionListItem>> {
    let (i, (head, tail)) = nom::sequence::pair(
        term,
        nom::multi::many0(nom::sequence::pair(character::one_of("+-/%"), term)),
    )(i)?;

    let head = ExpressionListItem::TermItem(head);
    let mut tail: Vec<ExpressionListItem> = tail
        .into_iter()
        .flat_map(|(op, term)| {
            let op = match op {
                '+' => NumericOperation::Add,
                '-' => NumericOperation::Subtract,
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
    let res = nom::combinator::recognize(nom::sequence::tuple((
        character::char(';'),
        character::not_line_ending,
    )))(i)?;

    Ok(res)
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
}
