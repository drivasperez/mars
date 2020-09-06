use crate::error::EvaluateError;
use crate::parser::Line;
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
}

fn from_lines(lines: Vec<Line>) -> Result<(), EvaluateError> {
    let mut metadata = Metadata::new();
    todo!()
}

fn get_metadata_from_line(line: &str) -> String {
    todo!()
}
