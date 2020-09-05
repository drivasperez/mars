use crate::parser::Line;
use crate::types::*;
use std::collections::HashMap;

fn evaluate_expressions(lines: Vec<Line>) -> Result<(), String> {
    let mut labels: HashMap<&str, usize> = HashMap::new();
    let instructions: Vec<(usize, &Line)> = lines
        .iter()
        .filter(|l| matches!(l, Line::Instruction(_)))
        .enumerate()
        .collect();

    for (i, line) in &instructions {
        if let Line::Instruction(instruction) = line {
            for label in &instruction.label_list {
                if let Some(_) = labels.insert(label, *i) {
                    return Err(format!("More than one definition of {}", label));
                }
            }
        }
    }

    todo!()
}
