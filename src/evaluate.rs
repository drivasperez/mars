use crate::error::EvaluateError;
use crate::parser::{numeric_expr::NumericExpr, Line};
use crate::types::*;
use std::collections::HashMap;

struct Metadata {
    author: Option<String>,
    date: Option<String>,
    strategy: Option<String>,
    version: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            author: None,
            date: None,
            strategy: None,
            version: None,
        }
    }
}

struct Warrior {
    metadata: Metadata,
    instructions: Vec<RawInstruction>,
    starts_at_line: usize,
}

fn lines_by_type<'a>(
    lines: Vec<Line<'a>>,
) -> (Vec<Instruction<'a>>, Vec<&str>, Vec<NumericExpr<'a>>) {
    let mut org_statements = Vec::new();
    let mut comments = Vec::new();
    let mut instructions = Vec::new();

    for line in lines {
        match line {
            Line::Comment(comment) => comments.push(comment),
            Line::OrgStatement(statement) => org_statements.push(statement),
            Line::Instruction(instruction) => instructions.push(instruction),
            Line::Definition(_, _) => {}
        }
    }
    (instructions, comments, org_statements)
}

fn from_lines(lines: Vec<Line>) -> Result<(), EvaluateError> {
    let mut metadata = Metadata::new();
    let (instructions, comments, org_statements) = lines_by_type(lines);

    todo!()
}

fn get_metadata_from_line(line: &str) -> String {
    todo!()
}

fn get_starting_line(
    orgs: &Vec<NumericExpr>,
    labels: &HashMap<&str, i32>,
) -> Result<usize, EvaluateError> {
    let starting_line = match orgs.len() {
        0 => 1,
        1 => orgs[0].evaluate(labels)?,
        _ => return Err(EvaluateError::MultipleOrgs),
    };

    Ok(starting_line as usize)
}
