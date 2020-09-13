use crate::{
    error::CoreError,
    warrior::{Instruction, Warrior},
};

use super::Core;
use std::collections::VecDeque;
#[derive(Debug)]
pub struct CoreBuilder {
    pub(super) core_size: usize,
    pub(super) cycles_before_tie: usize,
    pub(super) initial_instruction: InitialInstruction,
    pub(super) instruction_limit: usize,
    pub(super) maximum_number_of_tasks: usize,
    pub(super) minimum_separation: usize,
    pub(super) read_distance: usize,
    pub(super) write_distance: usize,
    pub(super) separation: Separation,
    pub(super) warriors: Vec<Warrior>,
}

impl Default for CoreBuilder {
    fn default() -> Self {
        Self {
            core_size: 8000,
            cycles_before_tie: 80_000,
            initial_instruction: InitialInstruction::Fixed(Instruction::default()),
            instruction_limit: 100,
            maximum_number_of_tasks: 8000,
            minimum_separation: 100,
            read_distance: 8000,
            write_distance: 8000,
            separation: Separation::Random,
            warriors: Vec::new(),
        }
    }
}

impl CoreBuilder {
    /// Creates a new instance of CoreBuilder with default parameters and no warriors.
    pub fn new() -> Self {
        CoreBuilder::default()
    }

    /// Sets the core's size. Core size is the number of instructions which make up the core
    /// during the battle.
    pub fn core_size(&mut self, core_size: usize) -> &mut Self {
        self.core_size = core_size;
        self
    }

    /// Sets the number of cycles that the match can last for before it is declared a tie.
    pub fn cycles_before_tie(&mut self, cycles_before_tie: usize) -> &mut Self {
        self.cycles_before_tie = cycles_before_tie;
        self
    }

    /// Sets the core's initial intruction. The initial instruction is that instruction which is preloaded
    /// into core prior to loading warriors.  In addition to loading
    /// an instruction such as "DAT #0, #0" into all of core, the
    /// initial instruction could be set to `Random`, meaning core
    /// instructions are filled with randomly generated instructions.
    pub fn initial_instruction(&mut self, initial_instruction: InitialInstruction) -> &mut Self {
        self.initial_instruction = initial_instruction;
        self
    }

    /// The maximum number of instructions allowed per warrior.
    pub fn instruction_limit(&mut self, instruction_limit: usize) -> &mut Self {
        self.instruction_limit = instruction_limit;
        self
    }

    /// Each warrior can spawn multiple additional tasks. This variable sets the maximum
    /// number of tasks allowed per warrior. In other words, this is the size of each warrior's task queue.
    pub fn maximum_number_of_tasks(&mut self, maximum_number_of_tasks: usize) -> &mut Self {
        self.maximum_number_of_tasks = maximum_number_of_tasks;
        self
    }

    /// The minimum number of instructions from the first instruction
    /// of one warrior to the first instruction of the next warrior.
    pub fn minimum_separation(&mut self, minimum_separation: usize) -> &mut Self {
        self.minimum_separation = minimum_separation;
        // Need to put some limit on this related to number of warriors.
        self
    }
    /// This is the range available for warriors to read information
    /// from core.  Attempts to read outside the limits of this range
    /// result in reading within the local readable range.  The range
    /// is centered on the current instruction.  Thus, a range of
    /// 500 limits reading to offsets of (-249 -> +250) from the
    /// currently executing instruction.  The read limit can therefore
    /// be considered a mini-core within core.  An attempt to read
    /// location PC+251 reads location PC-249 instead.  An attempt to
    /// read location PC+500 reads location PC instead.
    ///
    /// Read distance must be a factor of core size, otherwise the
    /// above defined behaviour is not guaranteed.
    pub fn read_distance(&mut self, read_distance: usize) -> &mut Self {
        self.read_distance = read_distance;
        self
    }

    /// The number of instructions from the first instruction of one
    /// warrior to the first instruction of the next warrior.
    /// Separation can be set to `Random`, meaning separations will be
    /// chosen randomly from those larger than the minimum separation.
    pub fn separation(&mut self, separation: Separation) -> &mut Self {
        self.separation = separation;
        self
    }

    /// This is the range available for warriors to write information
    /// to core.  Attempts to write outside the limits of this range
    /// result in writing within the local writable range.  The range
    /// is centered on the current instruction.  Thus, a range of 500
    /// limits writing to offsets of (-249 -> +250) from the
    /// currently executing instruction.  The write limit can
    /// therefore be considered a mini-core within core.  An attempt
    /// to write location PC+251 writes to location PC-249 instead.  
    /// An attempt to write to location PC+500 writes to location PC
    /// instead.
    ///
    /// Write distance must be a factor of core size, otherwise the
    /// above defined behaviour is not guaranteed.
    pub fn write_distance(&mut self, write_distance: usize) -> &mut Self {
        self.write_distance = write_distance;
        self
    }

    pub fn load_warriors(&mut self, warriors: &[Warrior]) -> Result<&mut Self, CoreError> {
        // TODO: Implement this properly, checking things like max length.
        for warrior in warriors {
            if warrior.len() > self.instruction_limit {
                return Err(CoreError::WarriorTooLong(
                    warrior.len(),
                    self.instruction_limit,
                    warrior.metadata.name().unwrap_or("Unnamed").to_owned(),
                ));
            }
            if warrior.is_empty() {
                return Err(CoreError::EmptyWarrior(
                    warrior.metadata.name().unwrap_or("Unnamed").to_owned(),
                ));
            };
        }

        Ok(self)
    }

    /// Build the core, consuming the `CoreBuilder` and returning a [`Core`](../struct.Core.html) struct.
    pub fn build(&self) -> Result<Core, CoreError> {
        let CoreBuilder {
            initial_instruction,
            separation,
            warriors,
            maximum_number_of_tasks,
            ..
        } = self;
        let mut core_instructions = vec![initial_instruction.clone().extract(); self.core_size];

        let mut offset = 0_usize;
        let separation = separation.clone().extract();

        let mut initial_offsets: Vec<usize> = warriors.iter().map(|w| w.starts_at_line).collect();
        for (i, warrior) in warriors.iter().enumerate() {
            initial_offsets[i] += offset;
            for instruction in &warrior.instructions {
                core_instructions[offset] = instruction.clone();
                offset += 1;
            }

            offset += separation;
        }

        let task_queues: Vec<VecDeque<usize>> = initial_offsets
            .iter()
            .map(|offset| {
                let mut v = VecDeque::with_capacity(*maximum_number_of_tasks);
                v.push_back(*offset);
                v
            })
            .collect();

        let warriors: Vec<&Warrior> = warriors.iter().collect();

        Ok(Core {
            core: self,
            instructions: core_instructions,
            task_queues,
            current_queue: 0,
            total_instructions: 0,
            living_warriors_count: warriors.len(),
        })
    }
}

/// The separation between warriors at the start of a match.
///
/// The number of instructions from the first instruction of one warrior to the first instruction of the next warrior.
/// If a core's separation is `Random`, separations will be chosen randomly from the set of numbers larger than the core's minimum separation.
#[derive(Debug, Clone)]
pub enum Separation {
    Random,
    Fixed(usize),
}

impl Separation {
    /// Extract the separation value if it's `Fixed`, or get a random `usize` if it's `Random`.
    pub fn extract(self) -> usize {
        match self {
            Self::Random => todo!(),
            Self::Fixed(f) => f,
        }
    }
}

/// The value to which the core's memory addresses are initialised
/// at the beginning of the match.
///
/// The initial instruction is that instruction which is preloaded
/// into core prior to loading warriors. If set to `Random`, core
/// instructions are filled with randomly generated instructions.
#[derive(Debug, Clone)]
pub enum InitialInstruction {
    Random,
    Fixed(Instruction),
}

impl InitialInstruction {
    /// Extract the initial instruction if it's `Fixed`, or get a random `Instruction` if it's `Random`.
    pub fn extract(self) -> Instruction {
        match self {
            Self::Random => todo!(),
            Self::Fixed(instr) => instr,
        }
    }
}
