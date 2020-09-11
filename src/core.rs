use crate::warrior::{Instruction, Warrior};
pub struct Core {
    /// Core size is the number of instructions which make up core
    /// during the battle.
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
