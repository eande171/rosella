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
    GreaterThanEq
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Let {
        variable_type: String,
        name: String,
        value: Expr,
    },
    If {
        condition_type: String,
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>
    },
    With {
        os: String,
        body: Vec<Stmt>,
    },
    While { 
        condition_type: String,
        condition: Expr,
        body: Vec<Stmt>,
    },
    Function {
        name: String,
        arguments: Option<Vec<Expr>>,
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
            Token::Function => Ok(self.parse_fn_stmt()?),
            Token::Let => Ok(self.parse_let_stmt()?),
            Token::If => Ok(self.parse_if_stmt()?),
            Token::With => Ok(self.parse_with_stmt()?),
            Token::While => Ok(self.parse_while_stmt()?),
            Token::RawInstruction => Ok(self.parse_raw_stmt()?),
            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt::Expression(expr))
                
                //Err(RosellaError::InvalidStatement(self.current_token().to_owned()))
                //panic!("Unhandled Statement: {:?}", self.current_token());
            }
        }
    }

    fn parse_fn_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::Function)?;

        let name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifer after 'fn'".to_string())),
        };
        self.advance();

        self.expect_token(&Token::LParen)?;
        
        let arguments = self.parse_arguments()?;

        self.expect_token(&Token::LBrace)?;

        let mut body: Vec<Stmt> = Vec::new();

        while self.current_token() != &Token::RBrace {
            body.push(self.parse_stmt()?);
        }

        self.expect_token(&Token::RBrace)?;

        Ok(Stmt::Function {
            name,
            arguments: if arguments.is_empty() { None } else { Some(arguments) },
            body,
        })
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::Let)?;

        let variable_type = match self.current_token() {
            Token::Identifier(variable_type) => variable_type.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifer (for variable type) after 'let'".to_string())),
        };
        self.advance();

        let name = match self.current_token() {
            Token::Identifier(name) => name.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifer after 'let'".to_string())),
        };
        self.advance();

        self.expect_token(&Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect_token(&Token::Semicolon)?;
        Ok(Stmt::Let { variable_type, name, value })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::If)?;

        let condition_type = match self.current_token() {
            Token::Identifier(condition_type) => condition_type.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifer (for comparison type) after 'if'".to_string())),
        };
        self.advance();

        self.expect_token(&Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(&Token::RParen)?;

        self.expect_token(&Token::LBrace)?;

        let mut then_branch: Vec<Stmt> = Vec::new();
        while self.current_token() != &Token::RBrace {
            then_branch.push(self.parse_stmt()?);
        }
        self.expect_token(&Token::RBrace)?;

        let else_branch = if self.current_token() == &Token::Else {
            self.advance();
            //Some(self.parse_else_branch()?)
        
            if self.current_token() == &Token::If {
                Some(vec![self.parse_if_stmt()?])
            } else {
                self.expect_token(&Token::LBrace)?;
                let mut else_branch: Vec<Stmt> = Vec::new();
                while self.current_token() != &Token::RBrace {
                    else_branch.push(self.parse_stmt()?);
                }
                self.expect_token(&Token::RBrace)?;
                Some(else_branch)
            }
        } else {
            None
        };

        Ok(Stmt::If { condition_type, condition, then_branch, else_branch })
    }

    fn parse_with_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::With)?;

        let os = match self.current_token() {
            Token::Identifier(os) => os.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifier after 'with'".to_string())),
        };
        self.advance();

        self.expect_token(&Token::LBrace)?;

        let mut body: Vec<Stmt> = Vec::new();
        while self.current_token() != &Token::RBrace {
            body.push(self.parse_stmt()?);
        }
        self.expect_token(&Token::RBrace)?;

        Ok(Stmt::With { os, body })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::While)?;

        let condition_type = match self.current_token() {
            Token::Identifier(condition_type) => condition_type.clone(),
            _ => return Err(RosellaError::ParseError("Expected identifer (for comparison type) after 'with'".to_string())),
        };
        self.advance();

        self.expect_token(&Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(&Token::RParen)?;

        self.expect_token(&Token::LBrace)?;

        let mut body: Vec<Stmt> = Vec::new();
        while self.current_token() != &Token::RBrace {
            body.push(self.parse_stmt()?);
        }
        self.expect_token(&Token::RBrace)?;

        Ok(Stmt::While { condition_type, condition, body })
    }

    fn parse_raw_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::RawInstruction)?;

        let mut instructions: Vec<Expr> = Vec::new();

        while self.current_token() != &Token::Semicolon && self.current_token() != &Token::EOF {
            instructions.push(self.parse_expression()?);
        }

        self.expect_token(&Token::Semicolon)?;

        Ok(Stmt::RawInstruction(instructions))
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
        let primary = match self.current_token() {
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
        };

        if let Token::Identifier(_) = self.peek_previous() {
            if let Token::LParen = self.current_token() {
                let name = match self.peek_previous() {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(RosellaError::ParseError("Expected identifer after 'fn'".to_string())),
                };

                self.advance();

                let args = self.parse_arguments()?;

                self.expect_token(&Token::Semicolon)?;
                
                Ok(Expr::Call { name, args })
            }
            else {
                primary
            }
        }
        else {
            primary
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expr>, RosellaError> {
        let mut arguments = Vec::new();

        if self.current_token() == &Token::RParen {
            self.advance();
            return Ok(arguments);
        }

        loop {
            arguments.push(self.parse_expression()?);

            match self.current_token() {
                &Token::Comma => {
                    self.advance();
                },
                &Token::RParen => {
                    self.advance();
                    break;
                },
                _ => return Err(RosellaError::ParseError("Expected ',' or ')' after argument".to_string()))
            }
        }

        Ok(arguments)
    }
}