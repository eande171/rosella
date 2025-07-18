use std::fmt::{self};
use std::error::Error;
use super::lexer::Token;

#[derive(Debug)]
pub enum RosellaError {
    InvalidPunctuation(Option<char>),
    InvalidToken(Option<char>),
    InvalidStatement(Token),
    UnexpectedToken(Token, Token),
}

impl fmt::Display for RosellaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RosellaError::InvalidPunctuation(punctuation) => write!(f, "Unhandled Punctuation: {:?}", punctuation),
            RosellaError::InvalidToken(token) => write!(f, "Input does not match a valid token: {:?}", token),
            RosellaError::InvalidStatement(statement) => write!(f, "Unhandled Statement: {:?}", statement),
            RosellaError::UnexpectedToken(expected_token, found_token) => write!(f, "Expected: {:?}, found: {:?}", expected_token, found_token)
        }
    }
}

impl Error for RosellaError {}