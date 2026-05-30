use std::{collections::VecDeque, rc::Rc};

use crate::{
    ast::{Expr, ExprKind, FunctionDeclaration, FunctionKind, Literal, Stmt},
    errors::LoxError,
    operator_type::{BinaryOp, BinaryOpType, LogicalOp, LogicalOpType, UnaryOp, UnaryOpType},
    token::Token,
    token_type::TokenType::{self, Identifier},
};

pub struct Parser {
    tokens: VecDeque<Token>,
    next_expr_id: usize,
}

// TODO: this whole thing could use less memory by actually consuming each token rather than leaving the vector intact
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into(),
            next_expr_id: 0,
        }
    }

    fn expr(&mut self, kind: ExprKind) -> Expr {
        let id = self.next_expr_id;
        self.next_expr_id += 1;
        Expr { id, kind }
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
                self.advance();
                self.var_declaration()
            }
            TokenType::Fun => {
                self.advance();
                Ok(Stmt::Function(Rc::new(
                    self.function(FunctionKind::Function)?,
                )))
            }
            TokenType::Class => {
                self.advance();
                self.class_declaration()
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
        let next_token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let identifier_token = match &next_token.token_type {
            TokenType::Identifier(_) => next_token,
            _ => {
                return Err(LoxError::SyntaxError {
                    token: next_token,
                    message: "Expected identifier".to_string(),
                });
            }
        };
        let equal_token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let initializer = match equal_token.token_type {
            TokenType::Equal => {
                self.advance();
                Some(self.expression()?)
            }
            _ => None,
        };
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var {
            name: identifier_token.lexeme.clone(),
            token: identifier_token,
            initializer,
        })
    }

    fn class_declaration(&mut self) -> Result<Stmt, LoxError> {
        let identifier_token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let name = match identifier_token.token_type {
            Identifier(name) => name,
            _ => {
                return Err(LoxError::SyntaxError {
                    token: identifier_token,
                    message: "Expect class name".to_string(),
                });
            }
        };
        self.consume(TokenType::LeftBrace, "Expect '{' before class body")?;
        let mut methods = vec![];
        loop {
            let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
            if matches!(token.token_type, TokenType::RightBrace) {
                self.advance();
                break;
            }
            methods.push(self.function(FunctionKind::Method)?)
        }
        Ok(Stmt::Class { name, methods })
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if let Some(token) = self.peek() {
            match token.token_type {
                TokenType::Print => {
                    self.advance();
                    return self.print_statement();
                }
                TokenType::LeftBrace => {
                    self.advance();
                    return Ok(Stmt::Block(self.block()?));
                }
                TokenType::If => {
                    self.advance();
                    return self.if_statement();
                }
                TokenType::While => {
                    self.advance();
                    return self.while_statement();
                }
                TokenType::For => {
                    self.advance();
                    return self.for_statement();
                }
                TokenType::Return => {
                    let keyword = self.advance_or_panic();
                    return self.return_statement(keyword);
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
                self.advance();
                None
            }
            TokenType::Var => {
                self.advance();
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };
        let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let condition = match token.token_type {
            TokenType::Semicolon => self.expr(ExprKind::Literal(Literal::Bool(true))),
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

    fn return_statement(&mut self, keyword: Token) -> Result<Stmt, LoxError> {
        let peeked = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let value = match peeked.token_type {
            TokenType::Semicolon => None,
            _ => Some(self.expression()?),
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return {
            token: keyword,
            value,
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let expression = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expression))
    }

    fn function(&mut self, kind: FunctionKind) -> Result<FunctionDeclaration, LoxError> {
        let token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let name = match token.token_type {
            TokenType::Identifier(name) => name,
            _ => {
                return Err(LoxError::SyntaxError {
                    token,
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
                let token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
                if params.len() >= 255 {
                    return Err(LoxError::SyntaxError {
                        token,
                        message: format!("Too many parameters for function {name}"),
                    });
                }
                let parameter_name = match token.token_type {
                    TokenType::Identifier(name) => name,
                    _ => {
                        return Err(LoxError::SyntaxError {
                            token,
                            message: format!("Expected parameter name"),
                        });
                    }
                };
                params.push(parameter_name);
                let token = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
                if token.token_type != TokenType::Comma {
                    break;
                }
                self.advance();
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameter list.")?;
        self.consume(
            TokenType::LeftBrace,
            format!("Expected '{{' before {kind} body"),
        )?;
        let body = self.block()?;
        Ok(FunctionDeclaration { name, params, body })
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after 'if' condition")?;
        let then_branch = Box::new(self.statement()?);
        let peeked = self.peek().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        let else_branch = match peeked.token_type {
            TokenType::Else => {
                self.advance();
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
            let equals = self.advance_or_panic();
            let value = Box::new(self.assignment()?);
            match expr.kind {
                ExprKind::Variable { name } => Ok(self.expr(ExprKind::Assign { name, value })),
                ExprKind::Get { object, token } => Ok(self.expr(ExprKind::Set {
                    object,
                    token,
                    value,
                })),
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
        while matches!(self.peek().map(|t| &t.token_type), Some(TokenType::Or)) {
            let token = self.advance_or_panic();
            let operator = LogicalOp {
                op_type: LogicalOpType::Or,
                token,
            };
            let right = Box::new(self.equality()?);
            expr = self.expr(ExprKind::Logical {
                left: Box::new(expr),
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;
        while matches!(self.peek().map(|t| &t.token_type), Some(TokenType::And)) {
            let token = self.advance_or_panic();
            let operator = LogicalOp {
                op_type: LogicalOpType::And,
                token,
            };
            let right = Box::new(self.equality()?);
            expr = self.expr(ExprKind::Logical {
                left: Box::new(expr),
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        fn equality_op(token: &Token) -> Option<BinaryOpType> {
            match token.token_type {
                TokenType::BangEqual => Some(BinaryOpType::NotEqual),
                TokenType::EqualEqual => Some(BinaryOpType::Equal),
                _ => None,
            }
        }
        let mut expr = self.comparison()?;
        while let Some(op_type) = self.peek().and_then(equality_op) {
            let token = self.advance_or_panic();
            let operator = BinaryOp { op_type, token };
            let right = Box::new(self.comparison()?);
            expr = self.expr(ExprKind::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        fn comparison_op_type(token: &Token) -> Option<BinaryOpType> {
            match token.token_type {
                TokenType::Greater => Some(BinaryOpType::GreaterThan),
                TokenType::GreaterEqual => Some(BinaryOpType::GreaterThanEqualTo),
                TokenType::Less => Some(BinaryOpType::LessThan),
                TokenType::LessEqual => Some(BinaryOpType::LessThanEqualTo),
                _ => None,
            }
        }
        let mut expr = self.term()?;
        while let Some(op_type) = self.peek().and_then(comparison_op_type) {
            let token = self.advance_or_panic();
            let operator = BinaryOp { op_type, token };
            let right = self.term()?;
            expr = self.expr(ExprKind::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxError> {
        fn term_op_type(token: &Token) -> Option<BinaryOpType> {
            match token.token_type {
                TokenType::Minus => Some(BinaryOpType::Subtract),
                TokenType::Plus => Some(BinaryOpType::Add),
                _ => None,
            }
        }
        let mut expr = self.factor()?;
        while let Some(op_type) = self.peek().and_then(term_op_type) {
            let token = self.advance_or_panic();
            let operator = BinaryOp { op_type, token };
            let right = self.factor()?;
            expr = self.expr(ExprKind::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, LoxError> {
        fn factor_op_type(token: &Token) -> Option<BinaryOpType> {
            match token.token_type {
                TokenType::Slash => Some(BinaryOpType::Divide),
                TokenType::Star => Some(BinaryOpType::Multiply),
                _ => None,
            }
        }
        let mut expr = self.unary()?;
        while let Some(op_type) = self.peek().and_then(factor_op_type) {
            let token = self.advance_or_panic();
            let operator = BinaryOp { op_type, token };
            let right = self.unary()?;
            expr = self.expr(ExprKind::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        let op_type = self.peek().and_then(|t| match t.token_type {
            TokenType::Bang => Some(UnaryOpType::Not),
            TokenType::Minus => Some(UnaryOpType::Negative),
            _ => None,
        });
        if let Some(op_type) = op_type {
            let token = self.advance_or_panic();
            let operator = UnaryOp { op_type, token };
            let right = self.unary()?;
            Ok(self.expr(ExprKind::Unary {
                operator,
                right: Box::new(right),
            }))
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
            match token.token_type {
                TokenType::LeftParen => {
                    self.advance();
                    expr = self.finish_call(expr)?;
                }
                TokenType::Dot => {
                    self.advance();
                    let token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
                    let name = match &token.token_type {
                        TokenType::Identifier(name) => name.clone(),
                        _ => {
                            return Err(LoxError::SyntaxError {
                                token,
                                message: "Expect property name after '.'".to_string(),
                            });
                        }
                    };
                    expr = self.expr(ExprKind::Get {
                        object: Box::new(expr),
                        token,
                    })
                }
                _ => {
                    break;
                }
            }
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
                        self.advance();
                        arguments.push(self.expression()?)
                    }
                    _ => break,
                };
            }
        };
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(self.expr(ExprKind::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        }))
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.peek().is_none() {
            return Err(LoxError::UnexpectedEndOfPhrase);
        }
        let token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
        match token.token_type {
            TokenType::False => Ok(self.expr(ExprKind::Literal(Literal::Bool(false)))),
            TokenType::True => Ok(self.expr(ExprKind::Literal(Literal::Bool(true)))),
            TokenType::Nil => Ok(self.expr(ExprKind::Literal(Literal::Nil))),
            TokenType::This => Ok(self.expr(ExprKind::This { token })),
            TokenType::Number(value) => Ok(self.expr(ExprKind::Literal(Literal::Number(value)))),
            TokenType::String(value) => Ok(self.expr(ExprKind::Literal(Literal::String(value)))),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                Ok(self.expr(ExprKind::Grouping {
                    expression: Box::new(expr),
                }))
            }
            TokenType::Identifier(name) => Ok(self.expr(ExprKind::Variable { name })),
            _ => Err(LoxError::SyntaxError {
                token,
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
            Ok(self.advance_or_panic())
        } else {
            let token = self.advance().ok_or(LoxError::UnexpectedEndOfPhrase)?;
            Err(LoxError::SyntaxError {
                token,
                message: message.into(),
            })
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        let peeked = self.peek();
        match peeked {
            None => false,
            Some(token) => token.token_type == token_type,
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn advance_or_panic(&mut self) -> Token {
        self.advance()
            .expect("Tried to advance, but no token was found.")
    }

    // TODO: should be able to delete this and check and just use advance and peek
    fn is_at_end(&self) -> bool {
        let peeked = self.peek();
        peeked.is_none()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens
            .front()
            .filter(|token| token.token_type != TokenType::Eof)
    }

    fn synchronize(&mut self) {
        let mut last_token = self.advance();
        loop {
            if let Some(token) = last_token
                && token.token_type == TokenType::Semicolon
            {
                return;
            }
            match self.peek() {
                None => return,
                Some(token) => {
                    if matches!(
                        token.token_type,
                        TokenType::Class
                            | TokenType::For
                            | TokenType::Fun
                            | TokenType::If
                            | TokenType::Print
                            | TokenType::Return
                            | TokenType::Var
                            | TokenType::While
                    ) {
                        return;
                    }
                }
            }
            last_token = self.advance();
        }
    }
}
