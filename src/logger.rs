use super::core::Core;
pub trait Logger: std::fmt::Debug {
    fn log(&self, current_game_state: &Core);
}

#[derive(Debug)]
pub struct DebugLogger {}

// TODO: Make this actually good
impl Logger for DebugLogger {
    fn log(&self, current_game_state: &Core) {
        println!("{:#?}", current_game_state);
    }
}
