mod corebuilder;
pub use corebuilder::*;

use crate::parser::instruction::Opcode;
use crate::warrior::{Instruction, Warrior};
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

pub struct Core<'a> {
    pub(crate) core: &'a CoreBuilder,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) task_queues: Vec<VecDeque<usize>>,
    pub(crate) current_queue: usize,
    pub(crate) total_instructions: usize,
    pub(crate) living_warriors_count: usize,
}

impl Core<'_> {
    pub fn run(&mut self) {
        loop {
            if let ExecutionOutcome::GameOver = self.run_once() {
                break;
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

        todo!();
    }
}
