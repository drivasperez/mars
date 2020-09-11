use crate::error::{Error, EvaluateError, MetadataError};
use crate::parser::instruction::{Address, AddressMode, Instruction, Modifier, Opcode, Operation};
use crate::parser::line::Line;
use crate::parser::{metadata::MetadataValue, numeric_expr::NumericExpr, replace_definitions};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
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
        let addr1 = (mode, expr.evaluate(labels, current_line as i32)?);
        let Address { mode, expr } = field_b.unwrap_or_default();
        let addr2 = (mode, expr.evaluate(labels, current_line as i32)?);

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

/// Metadata about a warrior, which can include its name, author, creation date, version and a summary of
/// its strategy.
#[derive(Debug)]
pub struct Metadata {
    name: Option<String>,
    author: Option<String>,
    date: Option<String>,
    strategy: Option<String>,
    version: Option<String>,
}

impl Metadata {
    /// The warrior's name.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// The name of the warrior's author.
    pub fn author(&self) -> Option<&String> {
        self.author.as_ref()
    }

    /// The publication date of the warrior.
    pub fn date(&self) -> Option<&String> {
        self.date.as_ref()
    }

    /// A description of the warrior's strategy.
    pub fn strategy(&self) -> Option<&String> {
        self.strategy.as_ref()
    }

    /// The warrior's version. This does not have to use any particular schema.
    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
    }
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

    fn insert_value(&mut self, line: MetadataValue) -> Result<(), MetadataError> {
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

                self.strategy = Some(String::from(strategy));
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct Warrior {
    pub(crate) metadata: Metadata,
    pub(crate) instructions: Vec<RawInstruction>,

    pub(crate) starts_at_line: usize,
}

impl Display for Warrior {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} by {} - {} lines",
            &self.metadata.name.as_deref().unwrap_or("Unnamed warrior"),
            &self.metadata.author.as_deref().unwrap_or("Unnamed author"),
            self.instructions.len()
        )
    }
}

impl Warrior {
    pub fn parse(input: &str) -> Result<Warrior, Error> {
        let input = replace_definitions(input).map_err(Error::Parse)?;
        let ls = crate::parser::parse(&input).map_err(Error::Parse)?;
        Self::from_lines(ls).map_err(Error::Evaluate)
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
        1 => orgs[0].evaluate(labels, 0)?,
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

    #[test]
    fn evaluate_dwarf_metadata() {
        let dwarf_str = include_str!("../warriors/dwarf.red");

        let warrior = Warrior::parse(&dwarf_str).unwrap();

        assert_eq!(warrior.metadata.name().unwrap(), "Dwarf");
        assert_eq!(warrior.metadata.author().unwrap(), "A. K. Dewdney");
        assert_eq!(warrior.metadata.version().unwrap(), "94.1");
        assert_eq!(warrior.metadata.date().unwrap(), "April 29, 1993");

        let bad_dwarf_str = include_str!("../warriors/bad_dwarf.red");

        Warrior::parse(&bad_dwarf_str).unwrap_err();
    }

    #[test]
    fn evaluate_dwarf_lines() {
        let dwarf_str = include_str!("../warriors/dwarf.red");
        let warrior = Warrior::parse(&dwarf_str).unwrap();

        assert_eq!(warrior.instructions.len(), 4);

        assert_eq!(format!("{}", warrior.instructions[0]), "DAT.F #0, #0");
        assert_eq!(format!("{}", warrior.instructions[1]), "ADD.AB #4, $-1");
        assert_eq!(format!("{}", warrior.instructions[2]), "MOV.AB #0, @-2");
        assert_eq!(format!("{}", warrior.instructions[3]), "JMP.A $-2, $0");
    }
}
