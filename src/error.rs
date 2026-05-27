use thiserror::Error;

/// Errors that can occur during argument parsing.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("missing value for argument `{0}`")]
    MissingValue(String),
    
    #[error("unknown parse error")]
    Unknown,
}
