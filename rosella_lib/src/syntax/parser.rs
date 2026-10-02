use super::ast::{BinaryOp, Expr, OS, Param, Statement, Stmt, Type};
use super::token::{SpannedToken, Token};
use crate::error::{RosellaError, Span};

// Stack Overflow Guard
const MAX_DEPTH: usize = 64;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    position: usize,
    depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser {
            tokens,
            position: 0,
            depth: 0,
        }
    }

    fn current_token(&self) -> &Token {
        match self.tokens.get(self.position) {
            Some(spanned) => &spanned.token,
            None => &Token::Eof,
        }
    }

    fn peek_token(&self) -> &Token {
        match self.tokens.get(self.position + 1) {
            Some(spanned) => &spanned.token,
            None => &Token::Eof,
        }
    }

    fn current_span(&self) -> Span {
        match self.tokens.get(self.position).or(self.tokens.last()) {
            Some(spanned) => spanned.span,
            None => Span::default(),
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn error(&self, message: String) -> RosellaError {
        RosellaError::ParseError(message, self.current_span())
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), RosellaError> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(RosellaError::UnexpectedToken(
                expected.to_owned(),
                self.current_token().to_owned(),
                self.current_span(),
            ))
        }
    }

    fn nested<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, RosellaError>,
    ) -> Result<T, RosellaError> {
        if self.depth >= MAX_DEPTH {
            return Err(self.error(format!("Nesting is deeper than the limit of {}", MAX_DEPTH)));
        }

        self.depth += 1;
        let result = parse(self);
        self.depth -= 1;

        result
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, RosellaError> {
        let mut statements: Vec<Statement> = Vec::new();

        while self.current_token() != &Token::Eof {
            statements.push(self.parse_stmt()?);
        }

        Ok(statements)
    }

    fn parse_stmt(&mut self) -> Result<Statement, RosellaError> {
        let span = self.current_span();
        let kind = self.parse_kind()?;
        Ok(Statement { kind, span })
    }

    fn parse_kind(&mut self) -> Result<Stmt, RosellaError> {
        match self.current_token() {
            Token::Function => self.parse_fn_stmt(),
            Token::Let => self.parse_let_stmt(),
            Token::If => self.parse_if_stmt(),
            Token::With => self.parse_with_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Break => {
                self.advance();
                self.expect_statement_end()?;
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                self.expect_statement_end()?;
                Ok(Stmt::Continue)
            }
            Token::RawInstruction(_) => self.parse_raw_stmt(),
            Token::Identifier(name) if self.peek_token() == &Token::Assign => {
                let name = name.clone();
                self.advance();
                self.advance();
                let value = self.parse_expression()?;
                self.expect_statement_end()?;
                Ok(Stmt::Assign {
                    variable_type: None,
                    name,
                    value,
                })
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect_statement_end()?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    // Report At Statement End
    fn expect_statement_end(&mut self) -> Result<(), RosellaError> {
        if self.current_token() == &Token::Semicolon {
            self.advance();
            return Ok(());
        }

        let span = match self
            .position
            .checked_sub(1)
            .and_then(|i| self.tokens.get(i))
        {
            Some(previous) => previous.span,
            None => self.current_span(),
        };
        Err(RosellaError::ParseError(
            "Expected ';' after this statement".to_string(),
            span,
        ))
    }

    fn parse_block(&mut self, context: &str) -> Result<Vec<Statement>, RosellaError> {
        let open_span = self.current_span();
        self.expect_token(&Token::LBrace)?;

        self.nested(|parser| {
            let mut body: Vec<Statement> = Vec::new();

            loop {
                match parser.current_token() {
                    Token::RBrace => break,
                    Token::Eof => {
                        return Err(parser.error(format!(
                            "Expected '}}' to close '{}' block opened at {}",
                            context, open_span
                        )));
                    }
                    _ => body.push(parser.parse_stmt()?),
                }
            }

            parser.advance();
            Ok(body)
        })
    }

    fn parse_fn_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::Function)?;

        // Optional Return Type
        let type_span = self.current_span();
        let first = self.parse_identifier("fn", "function name")?;
        let (return_type, name) = match self.current_token() {
            Token::Identifier(name) => {
                let name = name.clone();
                let return_type = Type::from_name(&first).ok_or_else(|| {
                    RosellaError::ParseError(
                        format!("Unknown return type '{}', expected int or str", first),
                        type_span,
                    )
                })?;
                self.advance();
                (Some(return_type), name)
            }
            _ => (None, first),
        };

        self.expect_token(&Token::LParen)?;
        let parameters = self.parse_parameters()?;
        let body = self.parse_block("fn")?;

        Ok(Stmt::Function {
            name,
            return_type,
            parameters,
            body,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::Return)?;

        if self.current_token() == &Token::Semicolon {
            self.advance();
            return Ok(Stmt::Return(None));
        }

        let value = self.parse_expression()?;
        self.expect_statement_end()?;
        Ok(Stmt::Return(Some(value)))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Param>, RosellaError> {
        let mut parameters = Vec::new();

        if self.current_token() == &Token::RParen {
            self.advance();
            return Ok(parameters);
        }

        loop {
            let type_span = self.current_span();
            let first = self.parse_identifier("fn", "parameter type")?;

            let Token::Identifier(name) = self.current_token() else {
                return Err(RosellaError::ParseError(
                    format!(
                        "Parameter '{}' needs a type, like int {} or str {}",
                        first, first, first
                    ),
                    type_span,
                ));
            };
            let name = name.clone();
            let param_type = Type::from_name(&first).ok_or_else(|| {
                RosellaError::ParseError(
                    format!("Unknown parameter type '{}', expected int or str", first),
                    type_span,
                )
            })?;
            self.advance();
            parameters.push(Param { name, param_type });

            match self.current_token() {
                Token::Comma => self.advance(),
                Token::RParen => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(self.error(format!(
                        "Expected ',' or ')' after parameter, found: {:?}",
                        self.current_token()
                    )));
                }
            }
        }

        Ok(parameters)
    }

    fn parse_type(&mut self, context: &str) -> Result<Type, RosellaError> {
        let span = self.current_span();
        let name = self.parse_identifier(context, "type")?;

        Type::from_name(&name).ok_or_else(|| {
            RosellaError::ParseError(
                format!("Unknown type '{}', expected int or str", name),
                span,
            )
        })
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::Let)?;

        let variable_type = self.parse_type("let")?;

        let name = self.parse_identifier("let", "variable name")?;

        self.expect_token(&Token::Assign)?;
        let value = self.parse_expression()?;
        self.expect_statement_end()?;
        Ok(Stmt::Let {
            variable_type,
            name,
            value,
        })
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::If)?;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block("if")?;

        let else_branch = if self.current_token() == &Token::Else {
            self.advance();
            if self.current_token() == &Token::If {
                let span = self.current_span();
                let kind = self.nested(|parser| parser.parse_if_stmt())?;
                Some(vec![Statement { kind, span }])
            } else {
                Some(self.parse_block("else")?)
            }
        } else {
            None
        };

        Ok(Stmt::If {
            resolved: None,
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_with_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::With)?;

        let os_span = self.current_span();
        let os = match self.parse_identifier("with", "OS type")?.as_str() {
            "windows" => OS::Windows,
            "linux" => OS::Linux,
            other => {
                return Err(RosellaError::ParseError(
                    format!(
                        "Invalid OS type '{}' in 'with' statement, expected 'windows' or 'linux'",
                        other
                    ),
                    os_span,
                ));
            }
        };

        let body = self.parse_block("with")?;

        Ok(Stmt::With { os, body })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::While)?;

        let condition = self.parse_expression()?;
        let body = self.parse_block("while")?;

        Ok(Stmt::While {
            resolved: None,
            condition,
            body,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, RosellaError> {
        self.expect_token(&Token::For)?;

        let variable_type = self.parse_type("for")?;
        let name = self.parse_identifier("for", "loop variable name")?;
        self.expect_token(&Token::In)?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block("for")?;

        Ok(Stmt::For {
            variable_type,
            name,
            iterable,
            resolved: None,
            body,
        })
    }

    fn parse_raw_stmt(&mut self) -> Result<Stmt, RosellaError> {
        let Token::RawInstruction(text) = self.current_token() else {
            return Err(self.error(format!(
                "Expected raw instruction, found: {:?}",
                self.current_token()
            )));
        };
        let text = text.clone();
        self.advance();

        Ok(Stmt::RawInstruction(text))
    }

    fn parse_expression(&mut self) -> Result<Expr, RosellaError> {
        self.nested(|parser| {
            parser.binary_expression(
                &[
                    &[Token::Or],
                    &[Token::And],
                    &[Token::Equal, Token::NotEqual],
                    &[
                        Token::GreaterThan,
                        Token::GreaterThanEq,
                        Token::LessThan,
                        Token::LessThanEq,
                    ],
                    &[Token::Plus, Token::Minus],
                    &[Token::Multiply, Token::Divide, Token::Modulo],
                ],
                0,
            )
        })
    }

    fn binary_expression(
        &mut self,
        precedence: &[&[Token]],
        level: usize,
    ) -> Result<Expr, RosellaError> {
        if level >= precedence.len() {
            return self.primary();
        }

        let mut expr = self.binary_expression(precedence, level + 1)?;
        let current_operators = precedence[level];

        while current_operators.contains(self.current_token()) {
            let operator = self.token_to_binary_op(self.current_token())?;
            self.advance();

            let right = self.binary_expression(precedence, level + 1)?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn token_to_binary_op(&self, token: &Token) -> Result<BinaryOp, RosellaError> {
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
            Token::Modulo => Ok(BinaryOp::Modulo),
            Token::And => Ok(BinaryOp::And),
            Token::Or => Ok(BinaryOp::Or),
            _ => Err(self.error(format!("{:?} is not a valid binary operator", token))),
        }
    }

    fn primary(&mut self) -> Result<Expr, RosellaError> {
        match self.current_token() {
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Ok(Expr::Number(num))
            }
            Token::Not => {
                self.advance();
                let operand = self.nested(|parser| parser.primary())?;
                Ok(Expr::Not(Box::new(operand)))
            }
            // Negative Number Literals
            Token::Minus => {
                self.advance();
                match self.current_token() {
                    Token::Number(n) => {
                        let num = -*n;
                        self.advance();
                        Ok(Expr::Number(num))
                    }
                    _ => Err(self.error(format!(
                        "Expected number after '-', found: {:?}",
                        self.current_token()
                    ))),
                }
            }
            Token::String(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expr::String(string))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.current_token() == &Token::LParen {
                    self.advance();
                    let args = self.parse_arguments()?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(self.error(format!("Unexpected token: {:?}", self.current_token()))),
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
                Token::Comma => {
                    self.advance();
                }
                Token::RParen => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(self.error(format!(
                        "Expected ',' or ')' after argument, found: {:?}",
                        self.current_token()
                    )));
                }
            }
        }

        Ok(arguments)
    }

    fn parse_identifier(&mut self, context: &str, reason: &str) -> Result<String, RosellaError> {
        let value = match self.current_token() {
            Token::Identifier(value) => value.clone(),
            other => {
                return Err(self.error(format!(
                    "Expected identifier ({}) after '{}', found: {:?}",
                    reason, context, other
                )));
            }
        };
        self.advance();

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Lexer;

    fn parse(input: &str) -> Result<Vec<Stmt>, RosellaError> {
        parse_statements(input).map(|statements| {
            statements
                .into_iter()
                .map(|statement| statement.kind)
                .collect()
        })
    }

    fn parse_statements(input: &str) -> Result<Vec<Statement>, RosellaError> {
        Parser::new(Lexer::new(input).tokenise()?).parse()
    }

    #[test]
    fn comments_inside_blocks() {
        let ast = parse("while int(x < 3) { /* note */ print(x); }").unwrap();
        assert!(matches!(&ast[0], Stmt::While { body, .. } if body.len() == 1));
    }

    #[test]
    fn calls_and_identifiers() {
        let ast = parse("echo(x); y;").unwrap();
        assert_eq!(
            ast,
            vec![
                Stmt::Expression(Expr::Call {
                    name: "echo".into(),
                    args: vec![Expr::Identifier("x".into())]
                }),
                Stmt::Expression(Expr::Identifier("y".into())),
            ]
        );
    }

    #[test]
    fn raw_instruction_is_text() {
        assert_eq!(
            parse("|> ls -la;").unwrap(),
            vec![Stmt::RawInstruction("ls -la".into())]
        );
    }

    #[test]
    fn operator_precedence() {
        let ast = parse("let int x = 1 + 2 * 3 > -4;").unwrap();
        let Stmt::Let { value, .. } = &ast[0] else {
            panic!()
        };
        let Expr::Binary {
            operator,
            left,
            right,
        } = value
        else {
            panic!()
        };
        assert_eq!(*operator, BinaryOp::GreaterThan);
        assert_eq!(**right, Expr::Number(-4));
        assert!(matches!(
            **left,
            Expr::Binary {
                operator: BinaryOp::Add,
                ..
            }
        ));
    }

    #[test]
    fn conditions_are_plain_expressions() {
        let ast = parse("if x < 1 { }\nwhile (y) { }").unwrap();
        assert!(matches!(
            &ast[0],
            Stmt::If {
                resolved: None,
                condition: Expr::Binary { .. },
                ..
            }
        ));
        assert!(matches!(
            &ast[1],
            Stmt::While {
                condition: Expr::Identifier(_),
                ..
            }
        ));
        assert!(parse("if int(x < 1) { }").is_ok());
    }

    #[test]
    fn parameters_need_types() {
        let ast = parse("fn f(int a, str b) { }").unwrap();
        let Stmt::Function { parameters, .. } = &ast[0] else {
            panic!()
        };
        assert_eq!(
            parameters,
            &vec![
                Param {
                    name: "a".into(),
                    param_type: Type::Int
                },
                Param {
                    name: "b".into(),
                    param_type: Type::Str
                },
            ]
        );
        let message = parse("fn f(a) { }").unwrap_err().to_string();
        assert!(
            message.contains("line 1, column 6: Parameter 'a' needs a type"),
            "{}",
            message
        );
        assert!(
            parse("fn f(float a) { }")
                .unwrap_err()
                .to_string()
                .contains("Unknown parameter type 'float'")
        );
        assert!(parse("fn f(file a) { }").is_err());
        assert!(
            parse("let float x = 1;")
                .unwrap_err()
                .to_string()
                .contains("Unknown type 'float'")
        );
        assert!(parse("let file x = 1;").is_err());
    }

    #[test]
    fn return_types_and_statements() {
        let ast =
            parse("fn int add(int a, int b) { return a + b; }\nfn greet() { return; }").unwrap();
        assert!(matches!(
            &ast[0],
            Stmt::Function { return_type: Some(Type::Int), name, body, .. }
                if name == "add" && matches!(body.as_slice(), [Statement { kind: Stmt::Return(Some(_)), .. }])
        ));
        assert!(matches!(
            &ast[1],
            Stmt::Function { return_type: None, body, .. } if matches!(body.as_slice(), [Statement { kind: Stmt::Return(None), .. }])
        ));
        assert!(
            parse("fn float add() { }")
                .unwrap_err()
                .to_string()
                .contains("Unknown return type 'float'")
        );
    }

    #[test]
    fn for_loops() {
        let ast =
            parse("for int i in range(0, 10) { }\nfor str f in files(\"*.txt\") { }").unwrap();
        assert!(matches!(
            &ast[0],
            Stmt::For { variable_type: Type::Int, name, iterable: Expr::Call { .. }, .. } if name == "i"
        ));
        assert!(matches!(
            &ast[1],
            Stmt::For {
                variable_type: Type::Str,
                ..
            }
        ));
        assert!(
            parse("for i in range(0, 1) { }")
                .unwrap_err()
                .to_string()
                .contains("Unknown type 'i'")
        );
    }

    #[test]
    fn assignment() {
        let ast = parse("x = x + 1;").unwrap();
        assert!(matches!(
            &ast[0],
            Stmt::Assign {
                variable_type: None,
                value: Expr::Binary { .. },
                ..
            }
        ));
        assert!(
            parse("x = 1")
                .unwrap_err()
                .to_string()
                .contains("Expected ';'")
        );
    }

    #[test]
    fn statements_need_semicolons() {
        let message = parse(
            "print(1);
print(2)
print(3);",
        )
        .unwrap_err()
        .to_string();
        assert!(
            message.contains("line 2, column 8: Expected ';'"),
            "{}",
            message
        );
    }

    #[test]
    fn with_takes_identifier() {
        let ast = parse("with windows { print(1); }").unwrap();
        assert!(matches!(
            &ast[0],
            Stmt::With {
                os: OS::Windows,
                ..
            }
        ));
        assert!(parse("with mac { }").is_err());
    }

    #[test]
    fn missing_brace_is_reported() {
        let message = parse("fn f() {\n  print(1);\n").unwrap_err().to_string();
        assert!(
            message.contains("Expected '}' to close 'fn' block opened at line 1, column 8"),
            "{}",
            message
        );
    }

    #[test]
    fn deep_nesting_errors() {
        let input = format!("print({}1{});", "(".repeat(10_000), ")".repeat(10_000));
        assert!(parse(&input).unwrap_err().to_string().contains("Nesting"));
        assert!(parse(&format!("print({}1{});", "(".repeat(30), ")".repeat(30))).is_ok());
    }
}
