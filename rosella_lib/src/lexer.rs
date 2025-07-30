use super::error::RosellaError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Function,
    Let,
    If,
    Else,
    With,                   // E.g. with "windows", with "linux" 
    While,

    // Identifier & Literals
    Number(f64),
    String(String),
    Identifier(String),
    
    // Operators
    Assign,                 // =
    Plus,                   // +
    Minus,                  // -
    Multiply,               // *
    Divide,                 // /
    Equal,                  // ==
    NotEqual,               // !=
    LessThan,               // <
    GreaterThan,            // >
    LessThanEq,             // <=
    GreaterThanEq,          // >=
    
    RawInstruction,         // |> 
    
    // Delimiters
    LParen,                 // (
    RParen,                 // )

    LBrace,                 // {
    RBrace,                 // }
    
    LBraceSquare,           // [
    RBraceSquare,           // ]
    
    Comma,                  // ,
    Semicolon,              // ;

    // Comments
    Comment,           // /*

    EOF
}
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_character: Option<char>
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let characters: Vec<char> = input.chars().collect();
        let current = characters.get(0).copied();

        Lexer {
            input: characters,
            position: 0,
            current_character: current
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_character = self.input.get(self.position).copied();
    }

    fn read_number(&mut self) -> f64 {
        let mut string: String = String::new();

        // Read Each Number
        while let Some(ch) = self.current_character {
            if ch.is_ascii_digit() || ch == '.' {
                string.push(ch);
                self.advance();
            } 
            else {
                break;
            }
        }

        // Parse Number
        let result: f64 = match string.parse() {
            Ok(res) => res,
            Err(_) => {
                eprintln!("Cannot parse number: {}", string);
                0.0
            }
        };

        result
    }

    fn read_string(&mut self) -> String {
        let mut string: String = String::new();

        // Skip Quote
        self.advance();

        while let Some(ch) = self.current_character {
            // Skip Last Quote
            if ch == '"' {
                self.advance();
                break;
            }

            string.push(ch);
            self.advance();
        }

        string
    }

    fn read_identifer(&mut self) -> String {
        let mut string: String = String::new();

        while let Some(ch) = self.current_character {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                string.push(ch);
                self.advance();
            }
            else {
                break;
            }
        }

        string
    }

    fn determine_keyword(&self, text: String) -> Token {
        match text.as_str() {
            "fn" => Token::Function,
            "let" => Token::Let,
            "if" => Token::If,
            "else" => Token::Else,
            "with" => Token::With,
            "while" => Token::While,
            _ => Token::Identifier(text.to_string())
        }
    }

    fn determine_punctuation(&mut self, current_char: Option<char>) -> Result<Token, RosellaError> {        
        self.advance();

        match current_char {
            Some('=') => {
                if self.current_character == Some('=') {
                    self.advance();
                    return Ok(Token::Equal)
                }
                Ok(Token::Assign) 
            }
            Some('+') => Ok(Token::Plus),
            Some('-') => Ok(Token::Minus),
            Some('*') => {
                Ok(Token::Multiply)
            },
            Some('/') => {
                if self.current_character == Some('*') {
                    self.consume_comment()?;
                    return Ok(Token::Comment);                    
                }
                else {
                    Ok(Token::Divide)
                }
            },

            Some('<') => {
                if self.current_character == Some('=') {
                    self.advance();
                    Ok(Token::LessThanEq)
                }
                else{
                    Ok(Token::LessThan) 
                }
            }
            Some('>') => {
                self.advance();
                if self.current_character == Some('=') {
                    self.advance();
                    Ok(Token::GreaterThanEq)
                }
                else {
                    Ok(Token::GreaterThan) 
                }
            }

            Some('(') => Ok(Token::LParen),
            Some(')') => Ok(Token::RParen),
            Some('{') => Ok(Token::LBrace),
            Some('}') => Ok(Token::RBrace),
            Some('[') => Ok(Token::LBraceSquare),
            Some(']') => Ok(Token::RBraceSquare),
            Some(',') => Ok(Token::Comma),
            Some(';') => Ok(Token::Semicolon),
            //Some(_) => panic!("Unhandled Punctuation: {:?}", current_char),
            Some(_) => Err(RosellaError::InvalidPunctuation(current_char)),
            None => Ok(Token::EOF)
        }
    }

    fn consume_comment(&mut self) -> Result<(), RosellaError> {
        self.advance(); // Skip the initial '*'
        while let Some(ch) = self.current_character {
            if ch == '*' {
                self.advance();
                if self.current_character == Some('/') {
                    self.advance();
                    return Ok(());
                }
            } else {
                self.advance();
            }
        }
        Err(RosellaError::ParseError("Expected */ to end comment".to_string()))
    }

    pub fn tokenise(&mut self) -> Result<Vec<Token>, RosellaError> {
        let mut tokens: Vec<Token> = Vec::new();

        loop {
            let token: Token = match self.current_character {
                // Handle Whitespace
                Some('\n') | Some('\t') | Some('\r') => {
                    self.advance();
                    continue;
                }
                Some(ch) if ch.is_ascii_whitespace() => {
                    self.advance();
                    continue;
                }

                Some(ch) if ch.is_ascii_digit() => Token::Number(self.read_number()),
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let ident = self.read_identifer();
                    self.determine_keyword(ident)
                },
                Some('"') => Token::String(self.read_string()),
                Some('!') => {
                    self.advance();
                    if self.current_character == Some('=') {
                        self.advance();
                        Token::NotEqual
                    }
                    else {
                        continue;
                    }
                }
                Some('|') => {
                    self.advance();
                    if self.current_character == Some('>') {
                        self.advance();
                        Token::RawInstruction
                    }
                    else {
                        continue;
                    }
                }
                Some(ch) if ch.is_ascii_punctuation() => match self.determine_punctuation(self.current_character) {
                    Ok(token) => token,
                    Err(e) => return Err(e),
                },
                Some(_) => Err(RosellaError::InvalidToken(self.current_character))?,
                //Some(_) => panic!("Input does not match a valid token: {:?}", self.current_character),

                None => Token::EOF
            };
            
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }
}
