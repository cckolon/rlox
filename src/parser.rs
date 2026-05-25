use crate::{
    ast::{Expr, FunctionDeclaration, FunctionKind, Literal, Stmt},
    errors::LoxError,
    operator_type::{BinaryOp, BinaryOpType, LogicalOp, LogicalOpType, UnaryOp, UnaryOpType},
    token::Token,
    token_type::TokenType,
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// TODO: this whole thing could use less memory by actually consuming each token rather than leaving the vector intact
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // TODO: maybe these should be a specific subtype of LoxError, like SyntaxError
    pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxError> {
        // TODO: return all the errors not just the top one
        let mut statements: Vec<Stmt> = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let result = match token.token_type {
            TokenType::Var => {
                self.advance()?;
                self.var_declaration()
            }
            TokenType::Fun => {
                self.advance()?;
                self.function(FunctionKind::Function)
            }
            _ => self.statement(),
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        // TODO: rework this to combine with consume. Can prob do with generic
        let peeked = self.peek();
        let token = peeked.ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let name = match &token.token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(LoxError::SyntaxError {
                    token: token.clone(),
                    message: "Expected identifier".to_string(),
                });
            }
        };
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let initializer = match token.token_type {
            TokenType::Equal => {
                self.advance()?;
                Some(self.expression()?)
            }
            _ => None,
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if let Some(token) = self.peek() {
            match token.token_type {
                TokenType::Print => {
                    self.advance()?;
                    return self.print_statement();
                }
                TokenType::LeftBrace => {
                    self.advance()?;
                    return Ok(Stmt::Block(self.block()?));
                }
                TokenType::If => {
                    self.advance()?;
                    return self.if_statement();
                }
                TokenType::While => {
                    self.advance()?;
                    return self.while_statement();
                }
                TokenType::For => {
                    self.advance()?;
                    return self.for_statement();
                }
                TokenType::Return => {
                    self.advance()?;
                    return self.return_statement();
                }
                _ => {}
            }
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'")?;
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let initializer = match token.token_type {
            TokenType::Semicolon => {
                self.advance()?;
                None
            }
            TokenType::Var => {
                self.advance()?;
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let condition = match token.token_type {
            TokenType::Semicolon => Expr::Literal(Literal::Bool(true)),
            _ => self.expression()?,
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let increment = match token.token_type {
            TokenType::RightParen => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses")?;
        let mut body = self.statement()?;

        if let Some(increment_expression) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment_expression)]);
        };
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };
        if let Some(initializer_statement) = initializer {
            body = Stmt::Block(vec![initializer_statement, body])
        };
        Ok(body)
    }

    // TODO: can probably combine these
    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expression))
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxError> {
        let keyword = self.previous()?;
        let peeked = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let value = match peeked.token_type {
            TokenType::Semicolon => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expression))
    }

    fn function(&mut self, kind: FunctionKind) -> Result<Stmt, LoxError> {
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let name = match &token.token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                name
            }
            _ => {
                return Err(LoxError::SyntaxError {
                    token: token.clone(),
                    message: format!("Expect {kind} name."),
                });
            }
        };
        self.consume(
            TokenType::LeftParen,
            format!("Expect '(' after {kind} name."),
        )?;
        let mut params = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                // TODO: can I avoid cloning the token here?
                let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?.clone();
                let parameter_name = match &token.token_type {
                    TokenType::Identifier(name) => {
                        self.advance()?;
                        // TODO: and the name here?
                        name.clone()
                    }
                    _ => {
                        return Err(LoxError::SyntaxError {
                            token,
                            message: format!("Expected parameter name"),
                        });
                    }
                };
                if params.len() >= 255 {
                    return Err(LoxError::SyntaxError {
                        token,
                        message: format!("Too many parameters for function {parameter_name}"),
                    });
                }
                params.push(parameter_name.clone());
                let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?.clone();
                if token.token_type != TokenType::Comma {
                    break;
                }
                self.advance()?;
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameter list.")?;
        self.consume(
            TokenType::LeftBrace,
            format!("Expected '{{' before {kind} body"),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function(FunctionDeclaration { name, params, body }))
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if' condition")?;
        let then_branch = Box::new(self.statement()?);
        let peeked = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let else_branch = match peeked.token_type {
            TokenType::Else => {
                self.advance()?;
                Some(Box::new(self.statement()?))
            }
            _ => None,
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after while")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition")?;
        let body = self.statement()?;
        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut statements = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let expr = self.or()?;
        if let Some(token) = self.peek()
            && token.token_type == TokenType::Equal
        {
            self.advance()?;
            let equals = self.previous()?;
            let value = self.assignment()?;
            match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(LoxError::SyntaxError {
                    token: equals,
                    message: "Invalid assignment target.".to_string(),
                }),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.and()?;
        fn or_op(token: &Token) -> Option<LogicalOp> {
            if token.token_type == TokenType::Or {
                Some(LogicalOp {
                    op_type: LogicalOpType::Or,
                    token: token.clone(),
                })
            } else {
                None
            }
        }
        while let Some(operator) = self.peek().and_then(or_op) {
            self.advance()?;
            let right = Box::new(self.and()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;
        fn and_op(token: &Token) -> Option<LogicalOp> {
            if token.token_type == TokenType::And {
                Some(LogicalOp {
                    op_type: LogicalOpType::And,
                    token: token.clone(),
                })
            } else {
                None
            }
        }
        while let Some(operator) = self.peek().and_then(and_op) {
            self.advance()?;
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        fn equality_op(token: &Token) -> Option<BinaryOp> {
            let op_type = match token.token_type {
                TokenType::BangEqual => BinaryOpType::NotEqual,
                TokenType::EqualEqual => BinaryOpType::Equal,
                _ => return None,
            };
            Some(BinaryOp {
                op_type,
                token: token.clone(),
            })
        }
        let mut expr = self.comparison()?;
        while let Some(operator) = self.peek().and_then(equality_op) {
            self.advance()?;
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        fn comparison_op(token: &Token) -> Option<BinaryOp> {
            let op_type = match token.token_type {
                TokenType::Greater => BinaryOpType::GreaterThan,
                TokenType::GreaterEqual => BinaryOpType::GreaterThanEqualTo,
                TokenType::Less => BinaryOpType::LessThan,
                TokenType::LessEqual => BinaryOpType::LessThanEqualTo,
                _ => return None,
            };
            Some(BinaryOp {
                op_type,
                token: token.clone(),
            })
        }
        let mut expr = self.term()?;
        while let Some(operator) = self.peek().and_then(comparison_op) {
            self.advance()?;
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        fn term_op(token: &Token) -> Option<BinaryOp> {
            let op_type = match token.token_type {
                TokenType::Minus => BinaryOpType::Subtract,
                TokenType::Plus => BinaryOpType::Add,
                _ => return None,
            };
            Some(BinaryOp {
                op_type,
                token: token.clone(),
            })
        }
        let mut expr = self.factor()?;
        while let Some(operator) = self.peek().and_then(term_op) {
            self.advance()?;
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        fn factor_op(token: &Token) -> Option<BinaryOp> {
            let op_type = match token.token_type {
                TokenType::Slash => BinaryOpType::Divide,
                TokenType::Star => BinaryOpType::Multiply,
                _ => return None,
            };
            Some(BinaryOp {
                op_type,
                token: token.clone(),
            })
        }
        let mut expr = self.unary()?;
        while let Some(operator) = self.peek().and_then(factor_op) {
            self.advance()?;
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        fn unary_op(token: &Token) -> Option<UnaryOp> {
            match token.token_type {
                TokenType::Bang => Some(UnaryOpType::Not),
                TokenType::Minus => Some(UnaryOpType::Negative),
                _ => None,
            }
            .and_then(|op_type| {
                Some(UnaryOp {
                    op_type,
                    token: token.clone(),
                })
            })
        }
        if let Some(operator) = self.peek().and_then(unary_op) {
            self.advance()?;
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.primary()?;
        loop {
            let peeked = self.peek();
            let token = match peeked {
                Some(token) => token,
                None => break,
            };
            if token.token_type != TokenType::LeftParen {
                break;
            }
            self.advance()?;
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxError> {
        let mut arguments: Vec<Expr> = vec![];
        if !self.check(TokenType::RightParen) {
            if arguments.len() >= 255 {
                return Err(LoxError::SyntaxError {
                    token: self
                        .peek()
                        .ok_or(LoxError::UnexpectedEndOfPhrase)?
                        .to_owned(),
                    message: "Can't have more than 255 arguments".to_string(),
                });
            };
            arguments.push(self.expression()?);
            loop {
                let peeked = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
                match peeked.token_type {
                    TokenType::Comma => {
                        self.advance()?;
                        arguments.push(self.expression()?)
                    }
                    _ => break,
                };
            }
        };
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.peek().is_none() {
            return Err(LoxError::UnexpectedEndOfPhrase);
        }
        let token = self.advance()?;
        match &token.token_type {
            TokenType::False => Ok(Expr::Literal(Literal::Bool(false))),
            TokenType::True => Ok(Expr::Literal(Literal::Bool(true))),
            TokenType::Nil => Ok(Expr::Literal(Literal::Nil)),
            TokenType::Number(value) => Ok(Expr::Literal(Literal::Number(value.clone()))),
            TokenType::String(value) => Ok(Expr::Literal(Literal::String(value.clone()))),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping {
                    expression: Box::new(expr),
                })
            }
            TokenType::Identifier(name) => Ok(Expr::Variable { name: name.clone() }),
            _ => Err(LoxError::SyntaxError {
                token: token.clone(),
                message: "Expect expression".to_string(),
            }),
        }
    }

    fn consume(
        &mut self,
        token_type: TokenType,
        message: impl Into<String>,
    ) -> Result<Token, LoxError> {
        if self.check(token_type) {
            self.advance()
        } else {
            let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?.clone();
            Err(LoxError::SyntaxError {
                token,
                message: message.into(),
            })
        }
    }

    fn previous(&self) -> Result<Token, LoxError> {
        let prev_token = self.tokens.get(self.current - 1);
        match prev_token {
            Some(token) => Ok(token.clone()),
            None => Err(LoxError::NoPreviousValue),
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            let peeked = self.peek();
            match peeked {
                None => false,
                Some(token) => token.token_type == token_type,
            }
        }
    }

    fn advance(&mut self) -> Result<Token, LoxError> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        let peeked = self.peek();
        peeked.is_none()
    }

    fn peek(&self) -> Option<&Token> {
        let current_token = self.tokens.get(self.current);
        match current_token {
            None => None,
            Some(token) => {
                if token.token_type == TokenType::Eof {
                    None
                } else {
                    Some(token)
                }
            }
        }
    }

    fn synchronize(&mut self) {
        self.advance()
            .expect("failed to advance while handling error");
        loop {
            if let Ok(token) = self.previous()
                && token.token_type == TokenType::Semicolon
            {
                return;
            }
            let peeked = self.peek();
            match peeked {
                None => {
                    break;
                }
                Some(token) => match token.token_type {
                    TokenType::Class
                    | TokenType::For
                    | TokenType::Fun
                    | TokenType::If
                    | TokenType::Print
                    | TokenType::Return
                    | TokenType::Var
                    | TokenType::While => {
                        break;
                    }
                    _ => {
                        self.advance()
                            .expect("failed to advance while handling error");
                    }
                },
            }
        }
    }
}
