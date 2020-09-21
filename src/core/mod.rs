mod corebuilder;
pub use corebuilder::*;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use crate::{
    logger::GameEvent, logger::Logger, parser::instruction::Modifier, parser::instruction::Opcode,
};
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
#[derive(Debug, Clone, PartialEq)]
struct CoreInstruction {
    opcode: Opcode,
    modifier: Modifier,
    mode_a: AddressMode,
    addr_a: usize,
    mode_b: AddressMode,
    addr_b: usize,
}

impl Display for CoreInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} {}{}, {}{}",
            self.opcode, self.modifier, self.mode_a, self.addr_a, self.mode_b, self.addr_b
        )
    }
}

fn keep_in_bounds(input: i64, m: usize) -> usize {
    let mut i: i64 = input;
    let m = i64::try_from(m).unwrap();

    while i < 0 {
        i += m as i64;
    }

    (i % m) as usize // Safe coercion, can't under/overflow because clamped between 0 and m.
}

impl CoreInstruction {
    fn from_instruction(instruction: Instruction, core_size: usize) -> Self {
        Self {
            opcode: instruction.opcode,
            modifier: instruction.modifier,
            mode_a: instruction.mode_a,
            addr_a: keep_in_bounds(instruction.addr_a, core_size),
            mode_b: instruction.mode_b,
            addr_b: keep_in_bounds(instruction.addr_b, core_size),
        }
    }
}

/// The outcome of a single match.
///
/// If only a single warrior remains in the match,
/// the match is counted as a win for that warrior. If the game's instruction counter
/// reaches its maximum value before a winner can be declared,
/// the match is a draw between all warriors that are still active.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MatchOutcome<'a> {
    Win(&'a Warrior),
    Draw(Vec<&'a Warrior>),
}

impl Display for MatchOutcome<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Win(warrior) => write!(f, "Match won by: {}", warrior),
            Self::Draw(_) => write!(f, "Match was a draw"),
        }
    }
}

#[derive(Debug)]
pub struct Core<'a> {
    core: &'a CoreBuilder,
    instructions: Vec<CoreInstruction>,
    task_queues: VecDeque<(&'a Warrior, VecDeque<usize>)>,
    current_queue: usize,
    cycle_count: usize,
    logger: Option<Box<dyn Logger>>,
}

impl Core<'_> {
    /// The core's current cycle count.
    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    /// Utility for calculating wrapped reads/writes based on core size and read/write distance.
    fn fold(ptr: usize, limit: usize, core_size: usize) -> usize {
        let mut result = ptr % limit;
        if result > (limit / 2) {
            result += core_size - limit;
        }

        result
    }

    pub fn run(&mut self) -> MatchOutcome {
        while let ExecutionOutcome::Continue = self.run_once() {
            if let Some(ref logger) = self.logger {
                logger.log(&self, GameEvent::Continue);
            }
        }

        let warriors: Vec<&Warrior> = self.task_queues.iter().map(|(w, _)| *w).collect();

        let outcome = match warriors.len() {
            1 => MatchOutcome::Win(warriors[0]),
            _ => MatchOutcome::Draw(warriors),
        };

        if let Some(ref logger) = self.logger {
            logger.log(self, GameEvent::GameOver(outcome.clone()));
        }

        outcome
    }

    fn decrement_address(ptr: usize, limit: usize) -> usize {
        if ptr == 0 {
            limit - 1
        } else {
            ptr - 1
        }
    }

    fn evaluate_operand(&mut self, mode: AddressMode, addr: usize, task: usize) -> usize {
        // println!("Evaluating: {} {} at task {}", mode, addr, task);
        match mode {
            AddressMode::Immediate => task,
            AddressMode::Direct => {
                Core::fold(addr + task, self.core.read_distance, self.core.core_size)
            }
            AddressMode::AFieldIndirect => {
                let next = Core::fold(addr + task, self.core.read_distance, self.core.core_size);
                let addr = self.instructions[next].addr_a;
                Core::fold(next + addr, self.core.read_distance, self.core.core_size)
            }
            AddressMode::BFieldIndirect => {
                let next = Core::fold(addr + task, self.core.read_distance, self.core.core_size);
                let addr = self.instructions[next].addr_b;
                Core::fold(next + addr, self.core.read_distance, self.core.core_size)
            }
            AddressMode::AFieldPredecrementIndirect => {
                let next = Core::fold(addr + task, self.core.read_distance, self.core.core_size);
                self.instructions[next].addr_a = Core::fold(
                    Core::decrement_address(
                        self.instructions[next].addr_a,
                        self.core.write_distance,
                    ),
                    self.core.read_distance,
                    self.core.core_size,
                );
                let addr = self.instructions[next].addr_a;
                Core::fold(next + addr, self.core.read_distance, self.core.core_size)
            }
            AddressMode::BFieldPredecrementIndirect => {
                let next = Core::fold(addr + task, self.core.read_distance, self.core.core_size);
                self.instructions[next].addr_b = Core::fold(
                    Core::decrement_address(
                        self.instructions[next].addr_b,
                        self.core.write_distance,
                    ),
                    self.core.write_distance,
                    self.core.core_size,
                );
                let addr = self.instructions[next].addr_b;
                Core::fold(next + addr, self.core.read_distance, self.core.core_size)
            }
            AddressMode::AFieldPostincrementIndirect => {
                let next = Core::fold(addr + task, self.core.read_distance, self.core.core_size);
                let addr = self.instructions[next].addr_a;
                self.instructions[next].addr_a = Core::fold(
                    self.instructions[next].addr_a + 1,
                    self.core.write_distance,
                    self.core.core_size,
                );
                Core::fold(next + addr, self.core.read_distance, self.core.core_size)
            }
            AddressMode::BFieldPostincrementIndirect => {
                let next = Core::fold(addr + task, self.core.read_distance, self.core.core_size);
                let addr = self.instructions[next].addr_b;
                self.instructions[next].addr_b = Core::fold(
                    self.instructions[next].addr_b + 1,
                    self.core.write_distance,
                    self.core.core_size,
                );
                Core::fold(next + addr, self.core.read_distance, self.core.core_size)
            }
        }
    }

    fn run_once(&mut self) -> ExecutionOutcome {
        let instruction_register: CoreInstruction;
        let source_register: CoreInstruction;
        let destination_register: CoreInstruction;

        let read_distance = self.core.read_distance;
        let write_distance = self.core.write_distance;
        let core_size = self.core.core_size;
        let fold_read = |x| Core::fold(x, read_distance, core_size);
        let fold_write = |x| Core::fold(x, write_distance, core_size);
        let decrement = |x| Core::decrement_address(x, write_distance);

        // Unwrap because this function won't be run when empty... Maybe this is not true.
        let mut current = self.task_queues.pop_front().unwrap();
        let current_queue = &mut current.1;

        // Get the task, killing the warrior if it has no tasks.
        let task = match current_queue.pop_front() {
            Some(v) => v,
            None => {
                if let Some(ref logger) = self.logger {
                    logger.log(self, GameEvent::WarriorKilled(current.0));
                }
                return if self.task_queues.len() == 0 {
                    self.task_queues.push_front(current);
                    ExecutionOutcome::GameOver
                } else {
                    ExecutionOutcome::Continue
                };
            }
        };

        // Copy the instruction pointed to by the task to the IR.
        instruction_register = self.instructions[fold_read(task)].clone();

        // Evaluate the IR's A operand and put the resolved instruction in the source register.
        let source_ptr = self.evaluate_operand(
            instruction_register.mode_a,
            instruction_register.addr_a,
            task,
        );
        source_register = self.instructions[source_ptr].clone();

        // Evaluate the IR's B operand and put the resolved instruction in the destination register.
        let destination_ptr = self.evaluate_operand(
            instruction_register.mode_b,
            instruction_register.addr_b,
            task,
        );

        destination_register = self.instructions[destination_ptr].clone();

        // println!("Instruction: {}", instruction_register);
        // println!(
        //     "Source {}: {}, Destination {}: {}",
        //     source_ptr, source_register, destination_ptr, destination_register
        // );

        match instruction_register.opcode {
            Opcode::Dat => {}
            Opcode::Mov => {
                match instruction_register.modifier {
                    Modifier::I => {
                        self.instructions[destination_ptr] = source_register;
                    }
                    Modifier::A => {
                        self.instructions[destination_ptr].addr_a = source_register.addr_a;
                    }
                    Modifier::B => {
                        self.instructions[destination_ptr].addr_b = source_register.addr_b;
                    }
                    Modifier::AB => {
                        self.instructions[destination_ptr].addr_b = source_register.addr_a;
                    }
                    Modifier::BA => {
                        self.instructions[destination_ptr].addr_a = source_register.addr_b;
                    }
                    Modifier::F => {
                        self.instructions[destination_ptr].addr_a = source_register.addr_a;
                        self.instructions[destination_ptr].addr_b = source_register.addr_b;
                    }
                    Modifier::X => {
                        self.instructions[destination_ptr].addr_b = source_register.addr_a;
                        self.instructions[destination_ptr].addr_a = source_register.addr_b;
                    }
                };
                current_queue.push_back(task + 1);
            }
            Opcode::Add => {
                match instruction_register.modifier {
                    Modifier::A => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a + source_register.addr_a,
                        );
                    }
                    Modifier::B => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b + source_register.addr_b,
                        );
                    }
                    Modifier::AB => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b + source_register.addr_a,
                        );
                    }
                    Modifier::BA => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a + source_register.addr_b,
                        );
                    }
                    Modifier::F | Modifier::I => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a + source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b + source_register.addr_b,
                        );
                    }
                    Modifier::X => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b + source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a + source_register.addr_b,
                        );
                    }
                }
                current_queue.push_back(task + 1);
            }
            Opcode::Sub => {
                match instruction_register.modifier {
                    Modifier::A => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a - source_register.addr_a,
                        );
                    }
                    Modifier::B => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b - source_register.addr_b,
                        );
                    }
                    Modifier::AB => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b - source_register.addr_a,
                        );
                    }
                    Modifier::BA => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a - source_register.addr_b,
                        );
                    }
                    Modifier::F | Modifier::I => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a - source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b - source_register.addr_b,
                        );
                    }
                    Modifier::X => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b - source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a - source_register.addr_b,
                        );
                    }
                }
                current_queue.push_back(task + 1)
            }
            Opcode::Mul => {
                match instruction_register.modifier {
                    Modifier::A => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a * source_register.addr_a,
                        );
                    }
                    Modifier::B => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b * source_register.addr_b,
                        );
                    }
                    Modifier::AB => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b * source_register.addr_a,
                        );
                    }
                    Modifier::BA => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a * source_register.addr_b,
                        );
                    }
                    Modifier::F | Modifier::I => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a * source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b * source_register.addr_b,
                        );
                    }
                    Modifier::X => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b * source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a * source_register.addr_b,
                        );
                    }
                }
                current_queue.push_back(task + 1)
            }
            Opcode::Div => {
                match instruction_register.modifier {
                    Modifier::A => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a / source_register.addr_a,
                        );
                    }
                    Modifier::B => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b / source_register.addr_b,
                        );
                    }
                    Modifier::AB => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b / source_register.addr_a,
                        );
                    }
                    Modifier::BA => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a / source_register.addr_b,
                        );
                    }
                    Modifier::F | Modifier::I => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a / source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b / source_register.addr_b,
                        );
                    }
                    Modifier::X => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b / source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a / source_register.addr_b,
                        );
                    }
                }
                current_queue.push_back(task + 1)
            }
            Opcode::Mod => {
                match instruction_register.modifier {
                    Modifier::A => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a % source_register.addr_a,
                        );
                    }
                    Modifier::B => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b % source_register.addr_b,
                        );
                    }
                    Modifier::AB => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b % source_register.addr_a,
                        );
                    }
                    Modifier::BA => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a % source_register.addr_b,
                        );
                    }
                    Modifier::F | Modifier::I => {
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a % source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b % source_register.addr_b,
                        );
                    }
                    Modifier::X => {
                        self.instructions[destination_ptr].addr_b = fold_write(
                            self.instructions[destination_ptr].addr_b % source_register.addr_a,
                        );
                        self.instructions[destination_ptr].addr_a = fold_write(
                            self.instructions[destination_ptr].addr_a % source_register.addr_b,
                        );
                    }
                }
                current_queue.push_back(task + 1)
            }
            Opcode::Jmp => current_queue.push_back(source_ptr),
            Opcode::Jmz => match instruction_register.modifier {
                Modifier::A | Modifier::BA => {
                    current_queue.push_back(if destination_register.addr_a == 0 {
                        source_ptr
                    } else {
                        task + 1
                    })
                }
                Modifier::B | Modifier::AB => {
                    current_queue.push_back(if destination_register.addr_b == 0 {
                        source_ptr
                    } else {
                        task + 1
                    })
                }
                _ => current_queue.push_back(
                    if destination_register.addr_a == 0 && destination_register.addr_b == 0 {
                        source_ptr
                    } else {
                        task + 1
                    },
                ),
            },
            Opcode::Jmn => match instruction_register.modifier {
                Modifier::A | Modifier::BA => {
                    current_queue.push_back(if destination_register.addr_a != 0 {
                        source_ptr
                    } else {
                        task + 1
                    })
                }
                Modifier::B | Modifier::AB => {
                    current_queue.push_back(if destination_register.addr_b != 0 {
                        source_ptr
                    } else {
                        task + 1
                    })
                }
                _ => current_queue.push_back(
                    if destination_register.addr_a != 0 && destination_register.addr_b != 0 {
                        source_ptr
                    } else {
                        task + 1
                    },
                ),
            },

            Opcode::Djn => match instruction_register.modifier {
                Modifier::A | Modifier::BA => {
                    self.instructions[destination_ptr].addr_a =
                        fold_write(decrement(self.instructions[destination_ptr].addr_a));
                    current_queue.push_back(if self.instructions[destination_ptr].addr_a != 0 {
                        source_ptr
                    } else {
                        task + 1
                    })
                }
                Modifier::B | Modifier::AB => {
                    self.instructions[destination_ptr].addr_b =
                        fold_write(decrement(self.instructions[destination_ptr].addr_b));
                    current_queue.push_back(if self.instructions[destination_ptr].addr_b != 0 {
                        source_ptr
                    } else {
                        task + 1
                    })
                }
                _ => {
                    self.instructions[destination_ptr].addr_a =
                        fold_write(decrement(self.instructions[destination_ptr].addr_a));
                    self.instructions[destination_ptr].addr_b =
                        fold_write(decrement(self.instructions[destination_ptr].addr_b));
                    current_queue.push_back(
                        if self.instructions[destination_ptr].addr_a != 0
                            && self.instructions[destination_ptr].addr_b != 0
                        {
                            source_ptr
                        } else {
                            task + 1
                        },
                    )
                }
            },
            Opcode::Seq => {
                let skip = match instruction_register.modifier {
                    Modifier::A => source_register.addr_a == destination_register.addr_a,
                    Modifier::B => source_register.addr_b == destination_register.addr_b,
                    Modifier::AB => source_register.addr_a == destination_register.addr_b,
                    Modifier::BA => source_register.addr_b == destination_register.addr_a,
                    Modifier::F => {
                        source_register.addr_a == destination_register.addr_a
                            && source_register.addr_b == destination_register.addr_b
                    }
                    Modifier::X => {
                        source_register.addr_a == destination_register.addr_b
                            && source_register.addr_b == destination_register.addr_a
                    }
                    Modifier::I => source_register == destination_register,
                };

                current_queue.push_back(if skip { task + 2 } else { task + 1 })
            }
            Opcode::Slt => {
                let skip = match instruction_register.modifier {
                    Modifier::A => source_register.addr_a < destination_register.addr_a,
                    Modifier::B => source_register.addr_b < destination_register.addr_b,
                    Modifier::AB => source_register.addr_a < destination_register.addr_b,
                    Modifier::BA => source_register.addr_b < destination_register.addr_a,
                    Modifier::F | Modifier::I => {
                        source_register.addr_a < destination_register.addr_a
                            && source_register.addr_b < destination_register.addr_b
                    }
                    Modifier::X => {
                        source_register.addr_a < destination_register.addr_b
                            && source_register.addr_b < destination_register.addr_a
                    }
                };

                current_queue.push_back(if skip { task + 2 } else { task + 1 })
            }

            Opcode::Sne => {
                let skip = match instruction_register.modifier {
                    Modifier::A => source_register.addr_a != destination_register.addr_a,
                    Modifier::B => source_register.addr_b != destination_register.addr_b,
                    Modifier::AB => source_register.addr_a != destination_register.addr_b,
                    Modifier::BA => source_register.addr_b != destination_register.addr_a,
                    Modifier::F => {
                        source_register.addr_a != destination_register.addr_a
                            || source_register.addr_b != destination_register.addr_b
                    }
                    Modifier::X => {
                        source_register.addr_a != destination_register.addr_b
                            || source_register.addr_b != destination_register.addr_a
                    }
                    Modifier::I => {
                        source_register.addr_a != destination_register.addr_a
                            || source_register.addr_b != destination_register.addr_b
                            || source_register.mode_a != destination_register.mode_a
                            || source_register.mode_b != destination_register.mode_b
                    }
                };

                current_queue.push_back(if skip { task + 2 } else { task + 1 })
            }

            Opcode::Spl => {
                current_queue.push_back(task + 1);
                if current_queue.len() < self.core.maximum_number_of_tasks {
                    current_queue.push_back(source_register.addr_a);
                }
            }
            Opcode::Nop => current_queue.push_back(task + 1),
        };

        self.current_queue = if self.current_queue == self.core.warriors.len() - 1 {
            0
        } else {
            self.current_queue + 1
        };

        self.task_queues.push_back(current);

        self.cycle_count += 1;
        if self.cycle_count >= self.core.cycles_before_tie {
            return ExecutionOutcome::GameOver;
        };

        ExecutionOutcome::Continue
    }
}

#[cfg(test)]
mod test;
