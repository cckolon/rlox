use crate::{errors::LoxError, token::Token, token_type::TokenType};

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?
        }
        Ok(self.tokens)
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let c = self.advance();
        match c {
            '(' => Ok(self.add_token(TokenType::LeftParen)),
            ')' => Ok(self.add_token(TokenType::RightParen)),
            '{' => Ok(self.add_token(TokenType::LeftBrace)),
            '}' => Ok(self.add_token(TokenType::RightBrace)),
            ',' => Ok(self.add_token(TokenType::Comma)),
            '.' => Ok(self.add_token(TokenType::Dot)),
            '-' => Ok(self.add_token(TokenType::Minus)),
            '+' => Ok(self.add_token(TokenType::Plus)),
            ';' => Ok(self.add_token(TokenType::Semicolon)),
            '*' => Ok(self.add_token(TokenType::Star)),
            '!' => {
                let token_match = self.token_match('=');
                self.add_token(if token_match {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                });
                Ok(())
            }
            '=' => {
                let token_match = self.token_match('=');
                self.add_token(if token_match {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                });
                Ok(())
            }
            '<' => {
                let token_match = self.token_match('=');
                self.add_token(if token_match {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                });
                Ok(())
            }
            '>' => {
                let token_match = self.token_match('=');
                self.add_token(if token_match {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                });
                Ok(())
            }
            '/' => {
                let token_match = self.token_match('/');
                if token_match {
                    // comment
                    loop {
                        if self.peek() == Some('\n') {
                            break;
                        }
                        if self.is_at_end() {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
                Ok(())
            }
            ' ' => Ok(()),
            '\r' => Ok(()),
            '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            '"' => self.string(),
            c => {
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_alphabetic() {
                    self.identifier()
                } else {
                    Err(LoxError::ScannerError {
                        line: self.line,
                        character: c,
                        message: "Bad input token".to_string(),
                    })
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let next_char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        next_char
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            self.source.chars().nth(self.current)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.chars().count()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.substring(self.start, self.current);
        self.tokens.append(&mut vec![Token {
            token_type,
            lexeme: text,
            line: self.line,
        }])
    }

    fn token_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.source.chars().nth(self.current) != Some(expected) {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn string(&mut self) -> Result<(), LoxError> {
        loop {
            let peeked = self.peek();
            if peeked == Some('"') {
                break;
            }
            if self.is_at_end() {
                break;
            }
            if peeked == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            let character = self
                .source
                .chars()
                .nth(self.current - 1)
                .expect("Can't get last char of file containing at least this token: \"");
            return Err(LoxError::ScannerError {
                line: self.line,
                character,
                message: "Unterminated string".to_string(),
            });
        }
        self.advance();
        let value = self.substring(self.start + 1, self.current - 1);
        self.add_token(TokenType::String(value));
        Ok(())
    }

    fn number(&mut self) -> Result<(), LoxError> {
        self.loop_through_digits();
        let peeked = self.peek();
        let peeked_next: Option<char> = self.peek_next();
        if peeked == Some('.')
            && let Some(c) = peeked_next
        {
            if c.is_ascii_digit() {
                // consume the dot
                self.advance();
                self.loop_through_digits();
            }
        }
        let literal_string = self.substring(self.start, self.current);
        let parsed_literal = literal_string.parse().expect("Failed to parse float");
        self.add_token(TokenType::Number(parsed_literal));
        Ok(())
    }

    fn substring(&self, start: usize, end: usize) -> String {
        self.source.chars().skip(start).take(end - start).collect()
    }

    fn loop_through_digits(&mut self) {
        loop {
            let peeked = self.peek();
            match peeked {
                None => {
                    break;
                }
                Some(c) => {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    self.advance()
                }
            };
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.chars().count() {
            None
        } else {
            self.source.chars().nth(self.current + 1)
        }
    }

    fn identifier(&mut self) -> Result<(), LoxError> {
        loop {
            let peeked = self.peek();
            if let Some(c) = peeked
                && (c.is_alphanumeric() || c == '_')
            {
                self.advance();
            } else {
                break;
            }
        }
        let text = self.substring(self.start, self.current);
        let keyword_type = self.match_keyword(&text);
        match keyword_type {
            Some(t) => {
                self.add_token(t);
            }
            None => {
                self.add_token(TokenType::Identifier(text));
            }
        }
        Ok(())
    }

    fn match_keyword(&self, keyword: &str) -> Option<TokenType> {
        match keyword {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}
