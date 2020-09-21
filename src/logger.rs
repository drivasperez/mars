use crate::{core::MatchOutcome, warrior::Warrior};

use super::core::Core;

pub enum GameEvent<'a> {
    WarriorKilled(&'a Warrior),
    GameOver(MatchOutcome<'a>),
    Continue,
}

pub trait Logger: std::fmt::Debug {
    fn log(&self, current_game_state: &Core, event: GameEvent);
}

#[derive(Debug)]
pub struct DebugLogger {}

impl DebugLogger {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DebugLogger {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Make this actually good
impl Logger for DebugLogger {
    fn log(&self, current_game_state: &Core, event: GameEvent) {
        match event {
            GameEvent::WarriorKilled(warrior) => {
                println!(
                    "Killing a warrior: {} after {} cycles",
                    warrior,
                    current_game_state.cycle_count()
                );
            }
            GameEvent::GameOver(outcome) => {
                println!(
                    "Game over! {} after {} cycles",
                    outcome,
                    current_game_state.cycle_count()
                );
            }
            _ => {}
        }
    }
}
