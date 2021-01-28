use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidExpressionError;

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression cannot be parsed")
    }
}

impl Error for InvalidExpressionError {}

impl From<Box<dyn Error>> for InvalidExpressionError {
    fn from(_: Box<dyn Error>) -> Self {
        InvalidExpressionError
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HoursOutOfBoundsError {
    pub input: u8,
}

impl fmt::Display for HoursOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "at least one value in the expression is out of range, must be between 0 and 23 inclusivelyi, is {}", self.input)
    }
}

impl Error for HoursOutOfBoundsError {}

#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateInputError;

impl fmt::Display for DuplicateInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains duplicates")
    }
}

impl Error for DuplicateInputError {}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownWeekdayError {
    pub input: String,
}

impl fmt::Display for UnknownWeekdayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the expression contains invalid weekday input: {}",
            self.input
        )
    }
}

impl Error for UnknownWeekdayError {}

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidWeekSpecError {
    pub input: String,
}

impl fmt::Display for InvalidWeekSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains invalid input for the week spec, must be either 'odd' or 'even', is: {}", self.input)
    }
}

impl Error for InvalidWeekSpecError {}
