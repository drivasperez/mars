mod corebuilder;
pub use corebuilder::*;

use crate::{logger::Logger, parser::instruction::Opcode};
use crate::{
    parser::instruction::AddressMode,
    warrior::{Instruction, Warrior},
};
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

    fn evaluate_instructions(
        &mut self,
        mode_a: AddressMode,
        addr_a: i32,
        mode_b: AddressMode,
        addr_b: i32,
    ) {
        // This pointer nonsense can probably be simplified after porting from the spec
        let instr_ref_a: usize;
        let instr_ref_b: usize;

        let read_ptr_a: usize;
        let write_ptr_a: usize;
        let read_ptr_b: usize;
        let write_ptr_b: usize;

        let post_increment_addr: usize;

        if let AddressMode::Immediate = mode_a {
            read_ptr_a = 0;
            write_ptr_a = 0;
        } else {
            read_ptr_a = self.fold_read(addr_a as usize);
            write_ptr_a = self.fold_write(addr_a as usize);

            // NOTE TO SELF: the spec mars doesn't have post-decrement or pre-increment, need to factor that in.
            match mode_a {
                AddressMode::Immediate => unreachable!(),
                AddressMode::Direct => {}
                AddressMode::AFieldIndirect => {}
                AddressMode::BFieldIndirect => {}
                AddressMode::AFieldPredecrementIndirect => {}
                AddressMode::BFieldPredecrementIndirect => {}
                AddressMode::AFieldPostincrementIndirect => {}
                AddressMode::BFieldPostincrementIndirect => {}
            };
        };
    }

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
