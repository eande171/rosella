mod builtins;
mod check;
mod codegen;
mod error;
mod syntax;

pub use codegen::{Compiler, Shell};
pub use error::{RosellaError, Span};
pub use syntax::{Lexer, OS, Parser};
