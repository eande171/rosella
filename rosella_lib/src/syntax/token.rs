use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Function,
    Let,
    If,
    Else,
    With, // E.g. with windows, with linux
    While,
    Return,
    Break,
    Continue,
    For,
    In,

    // Identifier & Literals
    Number(i64),
    String(String),
    Identifier(String),

    // Operators
    Assign,        // =
    Plus,          // +
    Minus,         // -
    Multiply,      // *
    Divide,        // /
    Equal,         // ==
    NotEqual,      // !=
    Modulo,        // %
    And,           // &&
    Or,            // ||
    Not,           // !
    LessThan,      // <
    GreaterThan,   // >
    LessThanEq,    // <=
    GreaterThanEq, // >=

    RawInstruction(String), // |> shell code;

    // Delimiters
    LParen, // (
    RParen, // )

    LBrace, // {
    RBrace, // }

    LBraceSquare, // [
    RBraceSquare, // ]

    Comma,     // ,
    Semicolon, // ;

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}
