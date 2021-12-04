use std::error::Error as StdError;
use std::fmt;

/// A global error type that encapsulates all other, more specific
/// error types.
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
            Self::EmptyExpression => write!(f, "the expression string must not be empty"),
            Self::Syntax(e) => e.fmt(f),
            Self::UnexpectedEndOfInput => write!(
                f,
                "the parser reached the end of the expression but expected more characters"
            ),
            Self::TimeParse(e) => write!(f, "failed to parse time: {}", e),
            Self::IndeterminateOffset(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {}

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
            "unexpected sequence of characters starting at position '{}', expected {}, got '{}'",
            self.position, self.expected, self.continues
        )
    }
}

impl StdError for SyntaxError {}
