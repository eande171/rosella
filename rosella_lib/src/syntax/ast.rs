use std::fmt;

use crate::error::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    String(String),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Not(Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    And,
    Or,
}

impl BinaryOp {
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::LessThan
                | BinaryOp::LessThanEq
                | BinaryOp::GreaterThan
                | BinaryOp::GreaterThanEq
        )
    }

    pub fn is_logical(self) -> bool {
        matches!(self, BinaryOp::And | BinaryOp::Or)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OS {
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
    Str,
    // Conditions Only
    Check,
}

impl Type {
    pub fn from_name(name: &str) -> Option<Type> {
        match name {
            "int" => Some(Type::Int),
            "str" => Some(Type::Str),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Str => write!(f, "str"),
            Type::Check => write!(f, "check"),
        }
    }
}

// Resolved Condition
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Compare {
        value_type: Type,
        left: Expr,
        operator: BinaryOp,
        right: Expr,
    },
    Check {
        check: String,
        args: Vec<Expr>,
    },
    Not(Box<Condition>),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
}

// Resolved Loop
#[derive(Debug, Clone, PartialEq)]
pub enum Iteration {
    Range { start: Expr, end: Expr, step: i64 },
    Files { pattern: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: Stmt,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Let {
        variable_type: Type,
        name: String,
        value: Expr,
    },
    Assign {
        variable_type: Option<Type>,
        name: String,
        value: Expr,
    },
    If {
        resolved: Option<Condition>,
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    With {
        os: OS,
        body: Vec<Statement>,
    },
    While {
        resolved: Option<Condition>,
        condition: Expr,
        body: Vec<Statement>,
    },
    Function {
        name: String,
        return_type: Option<Type>,
        parameters: Vec<Param>,
        body: Vec<Statement>,
    },
    For {
        variable_type: Type,
        name: String,
        iterable: Expr,
        resolved: Option<Iteration>,
        body: Vec<Statement>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    RawInstruction(String),
}
