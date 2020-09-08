use crate::error::{EvaluateError, MetadataError};
use crate::parser::{metadata::MetadataValue, numeric_expr::NumericExpr, Line};
use crate::types::*;
use std::collections::HashMap;

struct Metadata {
    name: Option<String>,
    author: Option<String>,
    date: Option<String>,
    strategy: Option<String>,
    version: Option<String>,
}

macro_rules! insert_once {
    ($field:expr, $value:expr, $error:path) => {{
        if let Some(_) = $field {
            return Err($error);
        };
        $field = Some($value);
    }};
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            name: None,
            author: None,
            date: None,
            strategy: None,
            version: None,
        }
    }

    pub fn insert_value(&mut self, line: MetadataValue) -> Result<(), MetadataError> {
        match line {
            MetadataValue::Author(author) => insert_once!(
                self.author,
                String::from(author),
                MetadataError::DuplicateAuthorDefinition
            ),
            MetadataValue::Date(date) => insert_once!(
                self.date,
                String::from(date),
                MetadataError::DuplicateDateDefinition
            ),
            MetadataValue::Version(version) => insert_once!(
                self.version,
                String::from(version),
                MetadataError::DuplicateVersionDefinition
            ),
            MetadataValue::Name(name) => insert_once!(
                self.name,
                String::from(name),
                MetadataError::DuplicateNameDefinition
            ),
            MetadataValue::Strategy(strategy) => {
                if let Some(ref mut strat) = self.strategy {
                    strat.push('\n');
                    strat.push_str(&strategy);
                };

                self.date = Some(String::from(strategy));
            }
        };
        Ok(())
    }
}

pub struct Warrior {
    metadata: Metadata,
    instructions: Vec<RawInstruction>,
    starts_at_line: usize,
}

fn lines_by_type<'a>(
    lines: Vec<Line<'a>>,
) -> (
    Vec<Instruction<'a>>,
    Vec<NumericExpr<'a>>,
    Vec<MetadataValue>,
) {
    let mut org_statements = Vec::new();
    let mut instructions = Vec::new();
    let mut metadata = Vec::new();

    for line in lines {
        match line {
            Line::OrgStatement(statement) => org_statements.push(statement),
            Line::Instruction(instruction) => instructions.push(instruction),
            Line::MetadataStatement(value) => metadata.push(value),
            _ => {}
        }
    }
    (instructions, org_statements, metadata)
}

impl Warrior {
    pub fn from_lines(lines: Vec<Line>) -> Result<Warrior, EvaluateError> {
        let mut metadata = Metadata::new();
        let (instructions, org_statements, metadata_values) = lines_by_type(lines);
        for line in metadata_values {
            metadata
                .insert_value(line)
                .map_err(EvaluateError::BadMetadata)?;
        }
        let definitions = get_label_definitions(&instructions)?;
        let starts_at_line = get_starting_line(&org_statements, &definitions)?;
        let instructions: Result<Vec<_>, _> = instructions
            .into_iter()
            .enumerate()
            .map(|(i, instruction)| RawInstruction::from_instruction(instruction, &definitions, i))
            .collect();
        let instructions = instructions?;

        Ok(Warrior {
            instructions,
            metadata,
            starts_at_line,
        })
    }
}

fn get_label_definitions<'a>(
    instructions: &Vec<Instruction<'a>>,
) -> Result<HashMap<&'a str, i32>, EvaluateError> {
    let mut definitions = HashMap::new();

    for (index, instruction) in instructions.iter().enumerate() {
        for label in &instruction.label_list {
            if definitions.contains_key(label) {
                return Err(EvaluateError::DuplicateLabelDefinition(String::from(
                    *label,
                )));
            } else {
                definitions.insert(*label, index as i32);
            }
        }
    }

    Ok(definitions)
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
