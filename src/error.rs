use std::error::Error;
use std::fmt;

/// A global error type that encapsulates all other, more specific
/// error types below.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidExpressionError {
    EmptyExpression,
    HoursOutOfBounds(HoursOutOfBoundsError),
    DuplicateInput,
    UnknownWeekday,
    InvalidWeekSpec,
    InvalidHourSpec,
    ParseHour,
}

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyExpression => EmptyExpressionError.fmt(f),
            Self::HoursOutOfBounds(e) => e.fmt(f),
            Self::DuplicateInput => DuplicateInputError.fmt(f),
            Self::UnknownWeekday => UnknownWeekdayError.fmt(f),
            Self::InvalidWeekSpec => InvalidWeekSpecError.fmt(f),
            Self::InvalidHourSpec => InvalidHourSpecError.fmt(f),
            Self::ParseHour => ParseHourError.fmt(f),
        }
    }
}

impl Error for InvalidExpressionError {}

/// Quite self-explanatory. The expression string literal must not be empty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyExpressionError;

impl fmt::Display for EmptyExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression string must not be empty")
    }
}

impl Error for EmptyExpressionError {}

/// This error indicates that the hour spec of an expression contains
/// invalid input that is not within the range of `(0..=23)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoursOutOfBoundsError {
    pub input: u8,
}

impl fmt::Display for HoursOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "at least one value in the expression is out of range, must be between 0 and 23 inclusively, is {}", self.input)
    }
}

impl Error for HoursOutOfBoundsError {}

/// This error points to duplicates in the expression, e.g. multiple
/// occurrences of the same weekday.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DuplicateInputError;

impl fmt::Display for DuplicateInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains duplicates")
    }
}

impl Error for DuplicateInputError {}

/// An error indicating that some word in the weekday spec of an
/// expression cannot be parsed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnknownWeekdayError;

impl fmt::Display for UnknownWeekdayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains invalid weekday input")
    }
}

impl Error for UnknownWeekdayError {}

/// An error indicating that the week spec of an expression contains
/// invalid input or does not adhere to the prescribed syntax.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidWeekSpecError;

impl fmt::Display for InvalidWeekSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains invalid input for the week spec")
    }
}

impl Error for InvalidWeekSpecError {}

/// An error indicating that the hour spec of an expression contains
/// invalid input or does not adhere to the prescribed syntax.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidHourSpecError;

impl fmt::Display for InvalidHourSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains invalid input for the hour spec")
    }
}

impl Error for InvalidHourSpecError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseHourError;

impl fmt::Display for ParseHourError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse part of the hour spec")
    }
}

impl Error for ParseHourError {}
