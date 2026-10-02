use super::token::{SpannedToken, Token};
use crate::error::{RosellaError, Span};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_character: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        // Strip Byte Order Mark
        let input = input.strip_prefix('\u{FEFF}').unwrap_or(input);
        let characters: Vec<char> = input.chars().collect();
        let current = characters.first().copied();

        Lexer {
            input: characters,
            position: 0,
            current_character: current,
            line: 1,
            column: 1,
        }
    }

    fn span(&self) -> Span {
        Span {
            line: self.line,
            column: self.column,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    fn advance(&mut self) {
        if self.current_character == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else if self.current_character.is_some() {
            self.column += 1;
        }

        self.position += 1;
        self.current_character = self.input.get(self.position).copied();
    }

    fn read_number(&mut self) -> Result<i64, RosellaError> {
        let start = self.span();
        let mut string: String = String::new();

        // Capture Decimals For The Error
        while let Some(ch) = self.current_character {
            let decimal = ch == '.' && self.peek().is_some_and(|next| next.is_ascii_digit());
            if ch.is_ascii_digit() || decimal {
                string.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Batch Uses 32 Bit Integers
        match string.parse::<i64>() {
            Ok(number) if number <= i32::MAX as i64 => Ok(number),
            _ => Err(RosellaError::InvalidNumber(string, start)),
        }
    }

    fn read_string(&mut self) -> Result<String, RosellaError> {
        let start = self.span();
        let mut string: String = String::new();

        // Skip Quote
        self.advance();

        while let Some(ch) = self.current_character {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(string);
                }
                // Keep Unknown Escapes
                '\\' if matches!(self.peek(), Some('"' | '\\' | 'n' | 't')) => {
                    self.advance();
                    string.push(match self.current_character {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        other => other.unwrap_or_default(),
                    });
                    self.advance();
                }
                _ => {
                    string.push(ch);
                    self.advance();
                }
            }
        }

        Err(RosellaError::UnterminatedString(start))
    }

    // Raw Line Until Last Semicolon
    fn read_raw_instruction(&mut self, start: Span) -> Result<Token, RosellaError> {
        let line_end = self.input[self.position..]
            .iter()
            .position(|&ch| ch == '\n')
            .map_or(self.input.len(), |offset| self.position + offset);
        let Some(terminator) = self.input[self.position..line_end]
            .iter()
            .rposition(|&ch| ch == ';')
        else {
            return Err(RosellaError::UnterminatedRaw(start));
        };

        let text: String = self.input[self.position..self.position + terminator]
            .iter()
            .collect();
        for _ in 0..=terminator {
            self.advance();
        }

        Ok(Token::RawInstruction(text.trim().to_string()))
    }

    fn read_identifier(&mut self) -> String {
        let mut string: String = String::new();

        while let Some(ch) = self.current_character {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                string.push(ch);
                self.advance();
            } else {
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
            "return" => Token::Return,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "for" => Token::For,
            "in" => Token::In,
            _ => Token::Identifier(text),
        }
    }

    fn determine_punctuation(&mut self, current_char: char) -> Result<Token, RosellaError> {
        let start = self.span();
        self.advance();

        match current_char {
            '=' => {
                if self.current_character == Some('=') {
                    self.advance();
                    return Ok(Token::Equal);
                }
                Ok(Token::Assign)
            }
            '!' => {
                if self.current_character == Some('=') {
                    self.advance();
                    return Ok(Token::NotEqual);
                }
                Ok(Token::Not)
            }
            '&' => {
                if self.current_character == Some('&') {
                    self.advance();
                    return Ok(Token::And);
                }
                Err(RosellaError::InvalidPunctuation('&', start))
            }
            '%' => Ok(Token::Modulo),
            '|' => {
                if self.current_character == Some('>') {
                    self.advance();
                    return self.read_raw_instruction(start);
                }
                if self.current_character == Some('|') {
                    self.advance();
                    return Ok(Token::Or);
                }
                Err(RosellaError::InvalidPunctuation('|', start))
            }
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Multiply),
            '/' => Ok(Token::Divide),
            '<' => {
                if self.current_character == Some('=') {
                    self.advance();
                    return Ok(Token::LessThanEq);
                }
                Ok(Token::LessThan)
            }
            '>' => {
                if self.current_character == Some('=') {
                    self.advance();
                    return Ok(Token::GreaterThanEq);
                }
                Ok(Token::GreaterThan)
            }

            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            '[' => Ok(Token::LBraceSquare),
            ']' => Ok(Token::RBraceSquare),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            _ => Err(RosellaError::InvalidPunctuation(current_char, start)),
        }
    }

    fn consume_comment(&mut self) -> Result<(), RosellaError> {
        let start = self.span();

        // Skip Comment Opener
        self.advance();
        self.advance();

        while let Some(ch) = self.current_character {
            self.advance();
            if ch == '*' && self.current_character == Some('/') {
                self.advance();
                return Ok(());
            }
        }

        Err(RosellaError::UnterminatedComment(start))
    }

    pub fn tokenise(&mut self) -> Result<Vec<SpannedToken>, RosellaError> {
        let mut tokens: Vec<SpannedToken> = Vec::new();

        loop {
            let span = self.span();
            let token: Token = match self.current_character {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                    continue;
                }
                Some('/') if self.peek() == Some('*') => {
                    self.consume_comment()?;
                    continue;
                }

                Some(ch) if ch.is_ascii_digit() => Token::Number(self.read_number()?),
                Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                    let ident = self.read_identifier();
                    self.determine_keyword(ident)
                }
                Some('"') => Token::String(self.read_string()?),
                Some(ch) if ch.is_ascii_punctuation() => self.determine_punctuation(ch)?,
                Some(ch) => return Err(RosellaError::InvalidToken(ch, span)),

                None => Token::Eof,
            };

            let is_eof = token == Token::Eof;
            tokens.push(SpannedToken { token, span });
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token> {
        Lexer::new(input)
            .tokenise()
            .unwrap()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn greater_than_keeps_next_character() {
        assert_eq!(
            tokens("a>b"),
            vec![
                Token::Identifier("a".into()),
                Token::GreaterThan,
                Token::Identifier("b".into()),
                Token::Eof
            ]
        );
        assert_eq!(tokens("a>=b")[1], Token::GreaterThanEq);
    }

    #[test]
    fn non_ascii_letter_is_an_error() {
        assert!(matches!(
            Lexer::new("é").tokenise(),
            Err(RosellaError::InvalidToken('é', _))
        ));
    }

    #[test]
    fn logic_operators() {
        assert_eq!(
            tokens("!a && b || c % d != e"),
            vec![
                Token::Not,
                Token::Identifier("a".into()),
                Token::And,
                Token::Identifier("b".into()),
                Token::Or,
                Token::Identifier("c".into()),
                Token::Modulo,
                Token::Identifier("d".into()),
                Token::NotEqual,
                Token::Identifier("e".into()),
                Token::Eof
            ]
        );
        assert!(matches!(
            Lexer::new("a & b").tokenise(),
            Err(RosellaError::InvalidPunctuation('&', _))
        ));
        assert!(matches!(
            Lexer::new("| x").tokenise(),
            Err(RosellaError::InvalidPunctuation('|', _))
        ));
    }

    #[test]
    fn raw_instructions_capture_the_line() {
        assert_eq!(
            tokens(
                "|> ls -la | grep x; echo (y);
z"
            ),
            vec![
                Token::RawInstruction("ls -la | grep x; echo (y)".into()),
                Token::Identifier("z".into()),
                Token::Eof
            ]
        );
        assert_eq!(
            tokens("{ |> ls; }"),
            vec![
                Token::LBrace,
                Token::RawInstruction("ls".into()),
                Token::RBrace,
                Token::Eof
            ]
        );
        assert!(matches!(
            Lexer::new("|> ls -la").tokenise(),
            Err(RosellaError::UnterminatedRaw(_))
        ));
    }

    #[test]
    fn numbers() {
        assert_eq!(tokens("12")[0], Token::Number(12));
        assert_eq!(tokens("2147483647")[0], Token::Number(2147483647));
        for invalid in ["1.5", "1.2.3", "2147483648"] {
            assert!(matches!(
                Lexer::new(invalid).tokenise(),
                Err(RosellaError::InvalidNumber(text, _)) if text == invalid
            ));
        }
        assert!(matches!(
            Lexer::new(".5").tokenise(),
            Err(RosellaError::InvalidPunctuation('.', _))
        ));
    }

    #[test]
    fn strings() {
        assert_eq!(
            tokens(r#""a\"b\\c\n\t\q""#)[0],
            Token::String("a\"b\\c\n\t\\q".into())
        );
        assert!(matches!(
            Lexer::new("\"abc").tokenise(),
            Err(RosellaError::UnterminatedString(_))
        ));
    }

    #[test]
    fn comments_are_skipped() {
        assert_eq!(
            tokens("a /* x ** y */ / b"),
            vec![
                Token::Identifier("a".into()),
                Token::Divide,
                Token::Identifier("b".into()),
                Token::Eof
            ]
        );
        assert_eq!(tokens("/**/")[0], Token::Eof);
        assert!(matches!(
            Lexer::new("/* x").tokenise(),
            Err(RosellaError::UnterminatedComment(_))
        ));
        assert!(matches!(
            Lexer::new("/*/").tokenise(),
            Err(RosellaError::UnterminatedComment(_))
        ));
    }

    #[test]
    fn spans_track_lines_and_columns() {
        let spanned = Lexer::new("\u{FEFF}let\n  x").tokenise().unwrap();
        assert_eq!(spanned[0].span, Span { line: 1, column: 1 });
        assert_eq!(spanned[1].span, Span { line: 2, column: 3 });
    }
}
