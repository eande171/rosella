use std::fmt::{self};
use std::error::Error;
use super::lexer::Token;

#[derive(Debug)]
pub enum RosellaError {
    InvalidPunctuation(Option<char>),
    InvalidToken(Option<char>),
    InvalidStatement(Token),
    UnexpectedToken(Token, Token),
    ParseError(String),
}

impl fmt::Display for RosellaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RosellaError::InvalidPunctuation(punctuation) => write!(f, "Unhandled Punctuation: {:?}", punctuation),
            RosellaError::InvalidToken(token) => write!(f, "Input does not match a valid token: {:?}", token),
            RosellaError::InvalidStatement(statement) => write!(f, "Unhandled Statement: {:?}", statement),
            RosellaError::UnexpectedToken(expected_token, found_token) => write!(f, "Expected: {:?}, found: {:?}", expected_token, found_token),
            RosellaError::ParseError(msg) => write!(f, "Error Occurred during Parsing: {}", msg)
        }
    }
}

impl Error for RosellaError {}