use std::{iter::Peekable, str::Chars};

use crate::{errors::LoxError, token::Token, token_type::TokenType};

pub struct Scanner<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token, LoxError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(first_char) = self.advance() else {
                return None;
            };
            let next_token_type = self.get_next_token_type(first_char);
            match next_token_type {
                Ok(Some(token_type)) => {
                    return Some(Ok(Token {
                        token_type,
                        line: self.line,
                    }));
                }
                Ok(None) => {}
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            chars: source.chars().peekable(),
            line: 1,
        }
    }

    fn get_next_token_type(&mut self, first_char: char) -> Result<Option<TokenType>, LoxError> {
        match first_char {
            '(' => Ok(Some(TokenType::LeftParen)),
            ')' => Ok(Some(TokenType::RightParen)),
            '{' => Ok(Some(TokenType::LeftBrace)),
            '}' => Ok(Some(TokenType::RightBrace)),
            ',' => Ok(Some(TokenType::Comma)),
            '.' => Ok(Some(TokenType::Dot)),
            '-' => Ok(Some(TokenType::Minus)),
            '+' => Ok(Some(TokenType::Plus)),
            ';' => Ok(Some(TokenType::Semicolon)),
            '*' => Ok(Some(TokenType::Star)),
            '!' => {
                if let Some(t) = self.chars.peek()
                    && *t == '='
                {
                    self.advance();
                    Ok(Some(TokenType::BangEqual))
                } else {
                    Ok(Some(TokenType::Bang))
                }
            }
            '=' => {
                if let Some(t) = self.chars.peek()
                    && *t == '='
                {
                    self.advance();
                    Ok(Some(TokenType::EqualEqual))
                } else {
                    Ok(Some(TokenType::Equal))
                }
            }
            '<' => {
                if let Some(t) = self.chars.peek()
                    && *t == '='
                {
                    self.advance();
                    Ok(Some(TokenType::LessEqual))
                } else {
                    Ok(Some(TokenType::Less))
                }
            }
            '>' => {
                if let Some(t) = self.chars.peek()
                    && *t == '='
                {
                    self.advance();
                    Ok(Some(TokenType::GreaterEqual))
                } else {
                    Ok(Some(TokenType::Greater))
                }
            }
            '/' => {
                if let Some(c) = self.chars.peek()
                    && c == &'/'
                {
                    // comment
                    loop {
                        let c = self.advance();
                        if c == Some('\n') || c == None {
                            break;
                        }
                    }
                    Ok(None)
                } else {
                    Ok(Some(TokenType::Slash))
                }
            }
            ' ' => Ok(None),
            '\r' => Ok(None),
            '\t' => Ok(None),
            '\n' => {
                self.line += 1;
                Ok(None)
            }
            '"' => Ok(Some(self.string()?)),
            c => {
                if c.is_ascii_digit() {
                    Ok(Some(self.number(c)?))
                } else if c.is_alphabetic() || c == '_' {
                    Ok(Some(self.identifier(c)?))
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

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn string(&mut self) -> Result<TokenType, LoxError> {
        let mut accumulator = "".to_string();
        loop {
            let peeked = self.chars.peek();
            if peeked == Some(&'"') {
                break;
            }
            if peeked.is_none() {
                return Err(LoxError::ScannerError {
                    line: self.line,
                    // TODO: this should really be the final character in the source
                    character: '"',
                    message: "Unterminated string".to_string(),
                });
            }
            if peeked == Some(&'\n') {
                self.line += 1;
            }
            accumulator.push(self.advance().unwrap());
        }
        self.advance();
        Ok(TokenType::String(accumulator))
    }

    fn number(&mut self, first_char: char) -> Result<TokenType, LoxError> {
        let before_dot = format!("{}{}", first_char, self.loop_through_digits());
        // TODO: can minimize peeking here
        let after_dot = if self.chars.peek() == Some(&'.') {
            self.advance();
            if let Some(c) = self.chars.peek()
                && c.is_ascii_digit()
            {
                Some(self.loop_through_digits())
            } else {
                None
            }
        } else {
            None
        };
        let literal_string = match after_dot {
            None => before_dot,
            Some(value) => format!("{}.{}", before_dot, value),
        };
        let parsed_literal = literal_string.parse().expect("Failed to parse float");
        Ok(TokenType::Number(parsed_literal))
    }

    fn loop_through_digits(&mut self) -> String {
        let mut accum = "".to_string();
        loop {
            let peeked = self.chars.peek();
            match peeked {
                None => {
                    break;
                }
                Some(c) => {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    accum.push(self.advance().unwrap());
                }
            };
        }
        accum
    }

    fn identifier(&mut self, first_char: char) -> Result<TokenType, LoxError> {
        let mut accumulator = first_char.to_string();
        loop {
            let peeked = self.chars.peek();
            if let Some(c) = peeked
                && (c.is_alphanumeric() || c == &'_')
            {
                let next = self.advance().unwrap();
                accumulator.push(next);
            } else {
                break;
            }
        }
        let keyword_type = self.match_keyword(&accumulator);
        match keyword_type {
            Some(t) => Ok(t),
            None => Ok(TokenType::Identifier(accumulator)),
        }
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
