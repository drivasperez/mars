use crate::{
    error::CoreError,
    warrior::{Instruction, Warrior},
};
pub struct Core {
    core_size: usize,
    cycles_before_tie: usize,
    initial_instruction: InitialInstruction,
    instruction_limit: usize,
    maximum_number_of_tasks: usize,
    minimum_separation: usize,
    read_distance: usize,
    write_distance: usize,
    separation: Separation,
    warriors: Vec<Warrior>,
}

impl Default for Core {
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

impl Core {
    /// Creates a new instance of Core with default parameters and no warriors.
    pub fn new() -> Self {
        Core::default()
    }

    /// Sets the core's size. Core size is the number of instructions which make the up core
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
        // TODO: Implement this, checking things like max length.
        Ok(self)
    }
}

/// The number of instructions from the first instruction of one warrior to the first instruction of the next warrior.
/// If a core's separation is `Random`, separations will be chosen randomly from the set of numbers larger than the core's minimum separation.
pub enum Separation {
    Random,
    Fixed(usize),
}

///The initial instruction is that instruction which is preloaded
/// into core prior to loading warriors. If set to `Random`, core
/// instructions are filled with randomly generated instructions.
pub enum InitialInstruction {
    Random,
    Fixed(Instruction),
}
