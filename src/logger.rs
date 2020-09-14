use super::core::Core;
pub trait Logger {
    fn log(current_game_state: &Core);
}

pub struct DefaultLogger;

// TODO: Make this actually good
impl Logger for DefaultLogger {
    fn log(current_game_state: &Core) {
        println!("{:#?}", current_game_state);
    }
}
