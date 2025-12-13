use crate::lexing::token::Token;
use crate::ast::{Expr, Stmt, BinOp};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    // ----------------- UTILITY ------------------

    fn current(&self) -> Result<&Token, ParseError> {
        self.tokens
            .get(self.position)
            .ok_or(ParseError::UnexpectedToken(
                "unexpected end of input".into(),
            ))
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        let token = self.current()?;

        let matches = match (expected, token) {
            ("semicolon", Token::SemiColon(_)) => true,
            ("left_paren", Token::LeftParen(_)) => true,
            ("right_paren", Token::RightParen(_)) => true,
            ("left_brace", Token::LeftBrace(_)) => true,
            ("right_brace", Token::RightBrace(_)) => true,
            _ => false,
        };

        if matches {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(format!(
                "expected {}, found {:?}",
                expected, token
            )))
        }
    }

    // ----------------- ENTRY POINT ------------------

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while self.position < self.tokens.len() {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    // ----------------- STATEMENTS ------------------

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self.current()?;

        match token {
            Token::Print(_) => self.parse_print(),
            Token::If(_) => self.parse_if(),
            Token::Int(_) => self.parse_var_declaration(),
            _ => Err(ParseError::UnexpectedToken(format!(
                "expected statement, found {:?}",
                token
            ))),
        }
    }

    fn parse_print(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume "print"

        self.expect("left_paren")?;
        let expr = self.parse_expression()?;
        self.expect("right_paren")?;
        self.expect("semicolon")?;

        Ok(Stmt::Print(expr))
    }

    fn parse_var_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume "int"

        let name = match self.current()? {
            Token::Identifier(id) => id.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken(
                    "expected identifier after int".into(),
                ))
            }
        };
        self.advance(); // consume identifier

        match self.current()? {
            Token::Assign(_) => self.advance(),
            _ => {
                return Err(ParseError::UnexpectedToken(
                    "expected '=' in variable declaration".into(),
                ))
            }
        };

        let value = self.parse_expression()?;
        self.expect("semicolon")?;

        Ok(Stmt::VarDeclaration { name, value })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume "if"

        self.expect("left_paren")?;
        let condition = self.parse_expression()?;
        self.expect("right_paren")?;

        self.expect("left_brace")?;
        let then_block = self.parse_block()?;
        self.expect("right_brace")?;

        let else_block = if matches!(self.current(), Ok(Token::Else(_))) {
            self.advance(); // consume "else"
            self.expect("left_brace")?;
            let block = self.parse_block()?;
            self.expect("right_brace")?;
            Some(block)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while let Ok(token) = self.current() {
            if matches!(token, Token::RightBrace(_)) {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    // ----------------- EXPRESSIONS ------------------

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_comparison()?;

        if matches!(self.current(), Ok(Token::Assign(_))) {
            self.advance(); // consume '='
            let value = self.parse_assignment()?; // right-associative

            if let Expr::Identifier(name) = left {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else {
                return Err(ParseError::UnexpectedToken(
                    "left side of assignment must be an identifier".into(),
                ));
            }
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;

        loop {
            let op = match self.current() {
                Ok(Token::GreaterThan(_)) => Some(BinOp::GreaterThan),
                Ok(Token::LessThan(_)) => Some(BinOp::LessThan),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_addition()?;

                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        loop {
            let op = match self.current() {
                Ok(Token::Plus(_)) => Some(BinOp::Add),
                Ok(Token::Minus(_)) => Some(BinOp::Sub),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_primary()?;

                left = Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
    // Get an owned Token so we don't hold a reference to self.tokens
    let token = self.current().cloned()?;
    self.advance();

    match token {
        Token::IntegerLiteral(n) => Ok(Expr::IntegerLiteral(n)),
        Token::StringLiteral(s) => Ok(Expr::StringLiteral(s)),
        Token::BooleanLiteral(b) => Ok(Expr::BooleanLiteral(b)),
        Token::Identifier(id) => Ok(Expr::Identifier(id)),

        Token::LeftParen(_) => {
            let expr = self.parse_expression()?;
            self.expect("right_paren")?;
            Ok(expr)
        }

        _ => Err(ParseError::UnexpectedToken(format!(
            "expected expression, found {:?}",
            token
        ))),
    }
}

}
