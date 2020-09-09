use crate::error::{Error, EvaluateError, MetadataError};
use crate::parser::instruction::{Address, AddressMode, Instruction, Modifier, Opcode, Operation};
use crate::parser::line::Line;
use crate::parser::{metadata::MetadataValue, numeric_expr::NumericExpr, replace_definitions};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub struct RawInstruction {
    opcode: Opcode,
    modifier: Modifier,
    addr_a: (AddressMode, i32),
    addr_b: (AddressMode, i32),
}

impl RawInstruction {
    pub fn new(
        opcode: Opcode,
        modifier: Modifier,
        addr_a: (AddressMode, i32),
        addr_b: (AddressMode, i32),
    ) -> Self {
        Self {
            opcode,
            modifier,
            addr_a,
            addr_b,
        }
    }
}

impl RawInstruction {
    pub(crate) fn from_instruction(
        instruction: Instruction,
        labels: &HashMap<&str, i32>,
        current_line: usize,
    ) -> Result<Self, EvaluateError> {
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
            self.opcode, self.modifier, self.addr_a.0, self.addr_a.1, self.addr_b.0, self.addr_b.1
        )
    }
}
struct Metadata {
    name: Option<String>,
    author: Option<String>,
    date: Option<String>,
    strategy: Option<String>,
    version: Option<String>,
}

macro_rules! insert_once {
    ($field:expr, $value:expr, $error:path) => {{
        if $field.is_some() {
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
    pub instructions: Vec<RawInstruction>,
    pub starts_at_line: usize,
}

impl Warrior {
    pub fn parse(input: &str) -> Result<Warrior, Error> {
        let input = replace_definitions(input).map_err(Error::Parse)?;
        let ls = crate::parser::parse(&input).map_err(Error::Parse)?;
        Self::from_lines(ls).map_err(Error::Evaluate)
    }

    /// The warrior's name.
    pub fn name(&self) -> &Option<String> {
        &self.metadata.name
    }

    /// The name of the warrior's author.
    pub fn author(&self) -> &Option<String> {
        &self.metadata.author
    }

    /// The publication date of the warrior.
    pub fn date(&self) -> &Option<String> {
        &self.metadata.date
    }

    /// A description of the warrior's strategy.
    pub fn strategy(&self) -> &Option<String> {
        &self.metadata.strategy
    }

    /// The warrior's version. This does not have to use any particular schema.
    pub fn version(&self) -> &Option<String> {
        &self.metadata.version
    }

    fn from_lines(lines: Vec<Line>) -> Result<Warrior, EvaluateError> {
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

fn get_label_definitions<'a>(
    instructions: &[Instruction<'a>],
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
    orgs: &[NumericExpr],
    labels: &HashMap<&str, i32>,
) -> Result<usize, EvaluateError> {
    let starting_line = match orgs.len() {
        0 => 1,
        1 => orgs[0].evaluate(labels)?,
        _ => return Err(EvaluateError::MultipleOrgs),
    };

    Ok(starting_line as usize)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::instruction::{AddressMode, Modifier, Opcode};
    #[test]
    fn display_raw_instruction() {
        let inst = RawInstruction {
            opcode: Opcode::Mov,
            modifier: Modifier::BA,
            addr_a: (AddressMode::Direct, 8),
            addr_b: (AddressMode::AFieldIndirect, 2),
        };

        assert_eq!(format!("{}", inst), String::from("MOV.BA $8, *2"));
    }
}
