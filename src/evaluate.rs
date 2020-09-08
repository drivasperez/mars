use crate::error::EvaluateError;
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

    fn insert_value(&mut self, line: MetadataValue) -> Result<(), EvaluateError> {
        match line {
            MetadataValue::Author(author) => self.insert_author(String::from(author))?,
            MetadataValue::Date(date) => self.insert_date(String::from(date))?,
            MetadataValue::Strategy(strategy) => self.insert_strategy(String::from(strategy)),
            MetadataValue::Version(version) => self.insert_version(String::from(version))?,
            MetadataValue::Name(name) => self.insert_name(String::from(name))?,
        }
        Ok(())
    }

    pub fn insert_author(&mut self, author: String) -> Result<(), EvaluateError> {
        if let Some(_) = self.author {
            return Err(EvaluateError::DuplicateAuthorDefinition);
        };

        self.author = Some(author);
        Ok(())
    }

    pub fn insert_name(&mut self, name: String) -> Result<(), EvaluateError> {
        if let Some(_) = self.name {
            return Err(EvaluateError::DuplicateNameDefinition);
        };

        self.name = Some(name);
        Ok(())
    }

    pub fn insert_date(&mut self, date: String) -> Result<(), EvaluateError> {
        if let Some(_) = self.date {
            return Err(EvaluateError::DuplicateDateDefinition);
        };

        self.date = Some(date);
        Ok(())
    }
    pub fn insert_version(&mut self, version: String) -> Result<(), EvaluateError> {
        if let Some(_) = self.version {
            return Err(EvaluateError::DuplicateVersionDefinition);
        };

        self.version = Some(version);
        Ok(())
    }
    pub fn insert_strategy(&mut self, strategy: String) {
        if let Some(ref mut strat) = self.strategy {
            strat.push('\n');
            strat.push_str(&strategy);
        };

        self.date = Some(strategy);
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
            Line::MetadataStatement(metadata) => todo!(),
            Line::Definition(_, _) => {}
        }
    }
    (instructions, comments, org_statements)
}

impl Warrior {
    pub fn from_lines(lines: Vec<Line>) -> Result<Warrior, EvaluateError> {
        // TODO: Get the metadata out the full-line
        let mut metadata = Metadata::new();
        let (instructions, comments, org_statements) = lines_by_type(lines);
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

fn get_metadata_from_line(line: &str) -> String {
    todo!()
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
