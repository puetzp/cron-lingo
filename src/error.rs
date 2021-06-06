use std::error::Error;
use std::fmt;

/// A global error type that encapsulates all other, more specific
/// error types below.
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidExpressionError {
    EmptyExpression,
    Syntax,
    DuplicateInput,
    UnknownWeekday,
    InvalidWeekSpec,
    InvalidWeekdaySpec,
    InvalidWeekdayModifier,
    InvalidHourSpec(time::ParseError),
    ParseHour,
}

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyExpression => EmptyExpressionError.fmt(f),
            Self::Syntax => SyntaxError.fmt(f),
            Self::DuplicateInput => DuplicateInputError.fmt(f),
            Self::UnknownWeekday => UnknownWeekdayError.fmt(f),
            Self::InvalidWeekSpec => InvalidWeekSpecError.fmt(f),
            Self::InvalidWeekdaySpec => InvalidWeekdaySpecError.fmt(f),
            Self::InvalidWeekdayModifier => InvalidWeekdayModifierError.fmt(f),
            Self::InvalidHourSpec(e) => e.fmt(f),
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

/// Generic syntax error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntaxError;

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression syntax is invalid")
    }
}

impl Error for SyntaxError {}

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

/// An error indicating that the weekday spec of an expression contains
/// invalid input or does not adhere to the prescribed syntax.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidWeekdaySpecError;

impl fmt::Display for InvalidWeekdaySpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the expression contains invalid input for the weekday spec"
        )
    }
}

impl Error for InvalidWeekdaySpecError {}

/// An error indicating that the weekday modifier of an expression contains
/// invalid input or does not adhere to the prescribed syntax.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidWeekdayModifierError;

impl fmt::Display for InvalidWeekdayModifierError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the expression contains an invalid weekday modifier, must be one of {{first, 1st, second, 2nd, third, 3rd, fourth, 4th}}"
        )
    }
}

impl Error for InvalidWeekdayModifierError {}

/// An error indicating that the hour spec of an expression contains
/// invalid input or does not adhere to the prescribed syntax.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidHourSpecError {
    pub source: time::ParseError,
}

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
