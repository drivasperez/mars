use thiserror::Error;
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Couldn't parse warrior")]
    Parse(nom::error::ErrorKind),
    #[error("Warrior incomplete")]
    Incomplete,
}
