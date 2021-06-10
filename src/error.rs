use std::error::Error;
use std::fmt;

/// A global error type that encapsulates all other, more specific
/// error types below.
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidExpressionError {
    EmptyExpression,
    Syntax,
    DuplicateInput,
    IllogicalWeekdayCombination,
    InvalidWeekSpec,
    InvalidWeekdaySpec,
    TimeParse(time::ParseError),
}

impl fmt::Display for InvalidExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyExpression => EmptyExpressionError.fmt(f),
            Self::Syntax => SyntaxError.fmt(f),
            Self::DuplicateInput => DuplicateInputError.fmt(f),
            Self::IllogicalWeekdayCombination => IllogicalWeekdayCombinationError.fmt(f),
            Self::InvalidWeekSpec => InvalidWeekSpecError.fmt(f),
            Self::InvalidWeekdaySpec => InvalidWeekdaySpecError.fmt(f),
            Self::TimeParse(e) => e.fmt(f),
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

/// This error points to illogical combinations of weekdays and its modifiers,
/// e.g. "on Mondays and the first Monday".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IllogicalWeekdayCombinationError;

impl fmt::Display for IllogicalWeekdayCombinationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the expression contains an illogical combination of weekdays"
        )
    }
}

impl Error for IllogicalWeekdayCombinationError {}

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

#[derive(Debug, Clone, PartialEq)]
pub struct TimeParseError {
    pub source: time::ParseError,
}

impl fmt::Display for TimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse a time specification")
    }
}

impl Error for TimeParseError {}
