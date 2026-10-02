use std::fmt;

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
}

impl BinaryOp {
    pub fn is_comparison(self) -> bool {
        !matches!(
            self,
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
        )
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
    File,
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
            Type::File => write!(f, "file"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
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
        condition_type: Option<Type>,
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    With {
        os: OS,
        body: Vec<Stmt>,
    },
    While {
        condition_type: Option<Type>,
        condition: Expr,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        return_type: Option<Type>,
        parameters: Vec<Param>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    RawInstruction(String),
}
