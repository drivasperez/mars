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
    #[error("Encountered divide by zero error")]
    DivideByZero,
}
