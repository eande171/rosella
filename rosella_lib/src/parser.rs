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

    fn peek_previous(&self) -> &Token {
        if self.position > 0 {
            match self.tokens.get(self.position - 1) {
                Some(token) => token,
                None => &Token::EOF
            }
        }
        else {
            &Token::EOF
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
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, RosellaError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while self.current_token() != &Token::EOF {
            statements.push(self.parse_stmt()?);
        }

        Ok(statements)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, RosellaError> {
        match self.current_token() {
            Token::Let => Ok(self.parse_let_stmt()?),
            _ => {
                Err(RosellaError::InvalidStatement(self.current_token().to_owned()))
                //panic!("Unhandled Statement: {:?}", self.current_token());
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::Let)?;

        let name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifer after 'let'".to_string())),
        };
        self.advance();

        self.expect_token(&Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect_token(&Token::Semicolon)?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_expression(&mut self) -> Result<Expr, RosellaError> {
        self.binary_expression(&[
            &[Token::Equal, Token::NotEqual],
            &[Token::GreaterThan, Token::GreaterThanEq, Token::LessThan, Token::LessThanEq],
            &[Token::Plus, Token::Minus],
            &[Token::Multiply, Token::Divide],
        ], 0)
    }

    fn binary_expression(&mut self, precedence: &[&[Token]], level: usize) -> Result<Expr, RosellaError> {
        if level >= precedence.len() {
            return self.primary();
        }

        let mut expr = self.binary_expression(precedence, level + 1)?;
        let current_operators = precedence[level];

        loop {
            if current_operators.contains(self.current_token()) {
                self.advance();
                let operator = self.token_to_binary_op(self.peek_previous().clone())?;

                let right = self.binary_expression(precedence, level + 1)?;

                expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) }
            }
            else {
                break;
            }
        }

        Ok(expr)
    }

    fn token_to_binary_op(&self, token: Token) -> Result<BinaryOp, RosellaError> {
        match token {
            Token::Equal => Ok(BinaryOp::Equal),
            Token::NotEqual => Ok(BinaryOp::NotEqual),
            Token::GreaterThan => Ok(BinaryOp::GreaterThan),
            Token::GreaterThanEq => Ok(BinaryOp::GreaterThanEq),
            Token::LessThan => Ok(BinaryOp::LessThan),
            Token::LessThanEq => Ok(BinaryOp::LessThanEq),
            Token::Plus => Ok(BinaryOp::Add),
            Token::Minus => Ok(BinaryOp::Subtract),
            Token::Multiply => Ok(BinaryOp::Multiply),
            Token::Divide => Ok(BinaryOp::Divide),
            _ => Err(RosellaError::ParseError(format!("{:?} is not a valid binary operator", token)))
        }
    }

    fn primary(&mut self) -> Result<Expr, RosellaError> {
        match self.current_token() {
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Expr::Number(num))
            }
            Token::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expr::String(string))
            }
            Token::Identifier(name) => {
                let variable_name = name.clone();
                self.advance();
                Ok(Expr::Identifier(variable_name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(RosellaError::ParseError(format!("Unexpected token: {:?}", self.current_token())))
        }
    }
}