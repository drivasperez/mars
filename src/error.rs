use thiserror::Error;
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Couldn't parse warrior")]
    Parse(nom::error::ErrorKind),
    #[error("Couldn't replace definitions")]
    Replace,
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
    #[error("Encountered divide by zero error")]
    DivideByZero,
    #[error("Encountered metadata error: {0}")]
    BadMetadata(MetadataError),
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Warrior defines name more than once")]
    DuplicateNameDefinition,
    #[error("Warrior defines version more than once")]
    DuplicateVersionDefinition,
    #[error("Warrior defines author more than once")]
    DuplicateAuthorDefinition,
    #[error("Warrior defines date more than once")]
    DuplicateDateDefinition,
}

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Encountered a warrior of zero length: {0}")]
    EmptyWarrior(String),
    #[error("Encountered a warrior of length {0} greater than max length {1}: {2}")]
    WarriorTooLong(usize, usize, String),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing warrior: {0}")]
    Parse(ParseError),
    #[error("Error evaluating warrior: {0}")]
    Evaluate(EvaluateError),
}
