use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidExpressionError;

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid expression")
    }
}

impl Error for InvalidExpressionError {}

impl From<Box<dyn Error>> for InvalidExpressionError {
    fn from(_: Box<dyn Error>) -> Self {
        InvalidExpressionError
    }
}
