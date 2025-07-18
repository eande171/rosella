use super::lexer::Token;
use super::error::RosellaError;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
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
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    GreaterThanEq
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Let {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>
    },
    With {
        os: String,
        body: Vec<Stmt>,
    },
    While { 
        condition: Expr,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        arguments: Vec<Expr>,
        body: Vec<Stmt>,
    },
    RawInstruction(Vec<Expr>)
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> &Token {
        match self.tokens.get(self.position) {
            Some(token) => token,
            None => &Token::EOF
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len(){
           self.position += 1;
        }
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), RosellaError> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        }
        else {
            Err(RosellaError::UnexpectedToken(expected.to_owned(), self.current_token().to_owned()))
            //lkpanic!("Expected: {:?}, found: {:?}", expected, self.current_token())
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, RosellaError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while self.current_token() != &Token::EOF {
            
        }

        Ok(statements)
    }

    pub fn parse_stmt(&mut self) -> Result<Stmt, RosellaError> {
        match self.current_token() {
            Token::Let => Ok(self.parse_let_stmt()?),
            _ => {
                Err(RosellaError::InvalidStatement(self.current_token().to_owned()))
                //panic!("Unhandled Statement: {:?}", self.current_token());
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, RosellaError> {
        Ok(Stmt::Expression(Expr::Number(0.0)))
    }
}