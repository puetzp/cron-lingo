use std::error::Error as StdError;
use std::fmt;

/// A global error type that encapsulates all other, more specific
/// error types below.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    EmptyExpression,
    Syntax(SyntaxError),
    UnexpectedEndOfInput,
    DuplicateInput,
    IllogicalWeekdayCombination,
    InvalidWeekSpec,
    InvalidWeekdaySpec,
    TimeParse(time::error::Parse),
    IndeterminateOffset(time::error::IndeterminateOffset),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyExpression => EmptyExpressionError.fmt(f),
            Self::Syntax(e) => e.fmt(f),
            Self::UnexpectedEndOfInput => UnexpectedEndOfInputError.fmt(f),
            Self::DuplicateInput => DuplicateInputError.fmt(f),
            Self::IllogicalWeekdayCombination => IllogicalWeekdayCombinationError.fmt(f),
            Self::InvalidWeekSpec => InvalidWeekSpecError.fmt(f),
            Self::InvalidWeekdaySpec => InvalidWeekdaySpecError.fmt(f),
            Self::TimeParse(e) => e.fmt(f),
            Self::IndeterminateOffset(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {}

/// Quite self-explanatory. The expression string literal must not be empty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyExpressionError;

impl fmt::Display for EmptyExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression string must not be empty")
    }
}

impl StdError for EmptyExpressionError {}

/// Generic syntax error. Gives the exact position of the erroneous character
/// in an expression and points out the expected input.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub(crate) position: usize,
    pub(crate) expected: String,
    pub(crate) continues: String,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "unexpected sequence of characters starting at position '{}', expected {}",
            self.position, self.expected
        )
    }
}

impl StdError for SyntaxError {}

/// This error indicates that the parser expected more input, but reached
/// the end of the expression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnexpectedEndOfInputError;

impl fmt::Display for UnexpectedEndOfInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "the parser reached the end of the input expression while expecting more characters"
        )
    }
}

impl StdError for UnexpectedEndOfInputError {}

/// This error points to duplicates in the expression, e.g. multiple
/// occurrences of the same weekday.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DuplicateInputError;

impl fmt::Display for DuplicateInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains duplicates")
    }
}

impl StdError for DuplicateInputError {}

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

impl StdError for IllogicalWeekdayCombinationError {}

/// An error indicating that the week spec of an expression contains
/// invalid input or does not adhere to the prescribed syntax.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidWeekSpecError;

impl fmt::Display for InvalidWeekSpecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the expression contains invalid input for the week spec")
    }
}

impl StdError for InvalidWeekSpecError {}

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

impl StdError for InvalidWeekdaySpecError {}
