use crate::syntax::Token;
use std::error::Error;
use std::fmt::{self};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug)]
pub enum RosellaError {
    InvalidPunctuation(char, Span),
    InvalidToken(char, Span),
    InvalidNumber(String, Span),
    UnterminatedString(Span),
    UnterminatedComment(Span),
    UnterminatedRaw(Span),
    UnexpectedToken(Token, Token, Span),
    ParseError(String, Span),
    CompilerError(String, Option<Span>),
}

impl fmt::Display for RosellaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RosellaError::InvalidPunctuation(punctuation, span) => {
                write!(f, "{}: Unhandled Punctuation: {:?}", span, punctuation)
            }
            RosellaError::InvalidToken(character, span) => write!(
                f,
                "{}: Input does not match a valid token: {:?}",
                span, character
            ),
            RosellaError::InvalidNumber(text, span) => {
                write!(
                    f,
                    "{}: {} is not a whole number that fits in an int",
                    span, text
                )
            }
            RosellaError::UnterminatedString(span) => {
                write!(f, "{}: Expected '\"' to end string", span)
            }
            RosellaError::UnterminatedComment(span) => {
                write!(f, "{}: Expected '*/' to end comment", span)
            }
            RosellaError::UnterminatedRaw(span) => write!(
                f,
                "{}: Expected ';' at the end of the raw instruction line",
                span
            ),
            RosellaError::UnexpectedToken(expected_token, found_token, span) => write!(
                f,
                "{}: Expected: {:?}, found: {:?}",
                span, expected_token, found_token
            ),
            RosellaError::ParseError(msg, span) => write!(f, "{}: {}", span, msg),
            RosellaError::CompilerError(msg, Some(span)) => write!(f, "{}: {}", span, msg),
            RosellaError::CompilerError(msg, None) => write!(f, "{}", msg),
        }
    }
}

impl RosellaError {
    pub(crate) fn compiler(message: impl Into<String>) -> Self {
        RosellaError::CompilerError(message.into(), None)
    }

    // Innermost Statement Wins
    pub(crate) fn located(self, span: Span) -> Self {
        match self {
            RosellaError::CompilerError(message, None) => {
                RosellaError::CompilerError(message, Some(span))
            }
            other => other,
        }
    }
}

impl Error for RosellaError {}
