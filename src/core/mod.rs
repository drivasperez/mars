mod corebuilder;
pub use corebuilder::*;

use crate::warrior::{Instruction, Warrior};
use crate::{logger::Logger, parser::instruction::Opcode};
use std::collections::VecDeque;

enum ExecutionOutcome {
    Continue,
    GameOver,
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
    instructions: Vec<Instruction>,
    task_queues: Vec<VecDeque<usize>>,
    current_queue: usize,
    total_instructions: usize,
    living_warriors_count: usize,
    logger: Option<Box<dyn Logger>>,
}

impl Core<'_> {
    pub fn run(&mut self) {
        while let ExecutionOutcome::Continue = self.run_once() {
            if let Some(ref logger) = self.logger {
                logger.log(&self);
            }
        }
    }

    fn run_once(&mut self) -> ExecutionOutcome {
        let current_queue = &mut self.task_queues[self.current_queue];
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

        let task = &self.instructions[task];

        let next_task: Option<usize> = match task.opcode {
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
