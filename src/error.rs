use std::error::Error as StdError;
use std::fmt;

/// A global error type that encapsulates all other, more specific
/// error types below.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyExpression,
    Syntax(SyntaxError),
    UnexpectedEndOfInput,
    TimeParse(time::error::Parse),
    IndeterminateOffset(time::error::IndeterminateOffset),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyExpression => EmptyExpressionError.fmt(f),
            Self::Syntax(e) => e.fmt(f),
            Self::UnexpectedEndOfInput => UnexpectedEndOfInputError.fmt(f),
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
            "unexpected sequence of characters starting at position '{}', expected {} while the expression continues '{}'",
            self.position, self.expected, self.continues
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
