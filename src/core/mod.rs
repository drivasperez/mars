mod corebuilder;
pub use corebuilder::*;
use std::convert::TryFrom;

use crate::{logger::Logger, parser::instruction::Modifier, parser::instruction::Opcode};
use crate::{
    parser::instruction::AddressMode,
    warrior::{Instruction, Warrior},
};
use std::collections::VecDeque;

enum ExecutionOutcome {
    Continue,
    GameOver,
}

/// Like a warrior instruction, but its addresses are positive 32-bit integers
#[derive(Debug, Clone)]
struct CoreInstruction {
    opcode: Opcode,
    modifier: Modifier,
    mode_a: AddressMode,
    addr_a: usize,
    mode_b: AddressMode,
    addr_b: usize,
}

fn keep_in_bounds(input: i64, offset: usize, m: usize) -> usize {
    let mut i: i64 = input;
    let m = i64::try_from(m).unwrap();
    let offset = i64::try_from(offset).unwrap();

    while i + offset < 0 {
        i += m as i64;
    }

    ((i + offset) % m) as usize // Safe coercion, can't under/overflow because clamped between 0 and m.
}

impl CoreInstruction {
    fn from_instruction(instruction: Instruction, current_offset: usize, core_size: usize) -> Self {
        Self {
            opcode: instruction.opcode,
            modifier: instruction.modifier,
            mode_a: instruction.mode_a,
            addr_a: keep_in_bounds(instruction.addr_a, current_offset, core_size),
            mode_b: instruction.mode_b,
            addr_b: keep_in_bounds(instruction.addr_b, current_offset, core_size),
        }
    }
}

/// The outcome of a single match.
///
/// If only a single warrior remains in the match,
/// the match is counted as a win for that warrior. If the game's instruction counter
/// reaches its maximum value before a winner can be declared,
/// the match is a draw between all warriors that are still active.
pub enum MatchOutcome<'a> {
    Win(&'a Warrior),
    Draw(Vec<&'a Warrior>),
}

#[derive(Debug)]
pub struct Core<'a> {
    core: &'a CoreBuilder,
    instructions: Vec<CoreInstruction>,
    task_queues: Vec<VecDeque<usize>>,
    current_queue: usize,
    total_instructions: usize,
    living_warriors_count: usize,
    logger: Option<Box<dyn Logger>>,
}

impl Core<'_> {
    /// Utility for calculating wrapped reads based on core size and read distance.
    fn fold_read(&self, ptr: usize) -> usize {
        let limit = self.core.read_distance;
        let mut result = ptr % limit;
        if result > (limit / 2) {
            result += self.core.core_size - limit;
        }

        result
    }

    /// Utility for calculating wrapped writes based on core size and write distance.
    fn fold_write(&self, ptr: usize) -> usize {
        let limit = self.core.write_distance;
        let mut result = ptr % limit;
        if result > (limit / 2) {
            result += self.core.core_size - limit;
        }

        result
    }

    pub fn run(&mut self) {
        while let ExecutionOutcome::Continue = self.run_once() {
            if let Some(ref logger) = self.logger {
                logger.log(&self);
            }
        }
    }

    fn evaluate_address(&self, mode: AddressMode, addr: usize) -> usize {
        todo!();
    }

    fn run_once(&mut self) -> ExecutionOutcome {
        let mut instruction_register: CoreInstruction;
        let mut source_register: CoreInstruction;
        let mut destination_register: CoreInstruction;

        let current_queue = &mut self.task_queues[self.current_queue];
        // Get the task, killing the warrior if it has no tasks.
        let task = match current_queue.pop_front() {
            Some(v) => v,
            None => {
                self.living_warriors_count -= 1;
                return if self.living_warriors_count == 0 {
                    ExecutionOutcome::GameOver
                } else {
                    ExecutionOutcome::Continue
                };
            }
        };

        // Copy the instruction the task
        instruction_register = self.instructions[task].clone();

        let next_task: Option<usize> = match instruction.opcode {
            Opcode::Dat => None,
            Opcode::Mov => todo!(),
            Opcode::Add => todo!(),
            Opcode::Sub => todo!(),
            Opcode::Mul => todo!(),
            Opcode::Div => todo!(),
            Opcode::Mod => todo!(),
            Opcode::Jmp => todo!(),
            Opcode::Jmz => todo!(),
            Opcode::Jmn => todo!(),
            Opcode::Djn => todo!(),
            Opcode::Slt => todo!(),
            Opcode::Seq => todo!(),
            Opcode::Sne => todo!(),
            Opcode::Spl => todo!(),
            Opcode::Nop => todo!(),
        };

        if let Some(x) = next_task {
            let current_queue = &mut self.task_queues[self.current_queue];
            current_queue.push_back(x);
        };

        self.current_queue = if self.current_queue == self.core.warriors.len() - 1 {
            0
        } else {
            self.current_queue + 1
        };

        self.total_instructions += 1;
        if self.total_instructions > self.core.instruction_limit {
            return ExecutionOutcome::GameOver;
        };

        ExecutionOutcome::Continue
    }
}
