use thiserror::Error;
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Couldn't parse warrior")]
    Parse(nom::error::ErrorKind),
    #[error("Warrior incomplete")]
    Incomplete,
}

#[derive(Error, Debug)]
pub enum EvaluateError {
    #[error("Encountered multiple org statements")]
    MultipleOrgs,
    #[error("Encountered use of undefined label: {0}")]
    UndefinedLabel(String),
    #[error("Encountered duplicate label definition: {0}")]
    DuplicateLabelDefinition(String),
    #[error("Warrior defines name more than once")]
    DuplicateNameDefinition,
    #[error("Warrior defines version more than once")]
    DuplicateVersionDefinition,
    #[error("Warrior defines author more than once")]
    DuplicateAuthorDefinition,
    #[error("Warrior defines date more than once")]
    DuplicateDateDefinition,
    #[error("Encountered divide by zero error")]
    DivideByZero,
}
