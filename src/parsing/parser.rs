use crate::lexing::token::Token;
use crate::parsing::ast::{Expr, Stmt, BinOp};


pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ----------------- utilities -----------------

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken)
        }
    }

    // ----------------- entry -----------------

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        while self.pos < self.tokens.len() {
            stmts.push(self.parse_stmt()?);
        }

        Ok(vec![Stmt::Block(stmts)])
    }

    // ----------------- statements -----------------

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.current() {
            Some(Token::Print) => self.parse_print(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Int) => self.parse_var_decl(),
            Some(Token::LeftBrace) => self.parse_block_stmt(),
            Some(Token::Paywall) => self.parse_paywall(),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    fn parse_block_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::LeftBrace)?;

        let mut stmts = Vec::new();
        while self.current() != Some(&Token::RightBrace) {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(Token::RightBrace)?;
        Ok(Stmt::Block(stmts))
    }

    fn parse_print(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'print'
        self.expect(Token::LeftParen)?;
        let expr = self.parse_expr()?;
        self.expect(Token::RightParen)?;
        self.expect(Token::SemiColon)?;
        Ok(Stmt::Print(expr))
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'int'

        let name = match self.current() {
            Some(Token::Identifier(id)) => {
                let n = id.clone();
                self.advance();
                n
            }
            _ => return Err(ParseError::UnexpectedToken),
        };

        self.expect(Token::Assign)?;
        let value = self.parse_expr()?;
        self.expect(Token::SemiColon)?;

        Ok(Stmt::VarDeclaration { name, value })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'if'
        self.expect(Token::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(Token::RightParen)?;

        let then_block = self.parse_block_stmt()?;

        let else_block = if self.current() == Some(&Token::Else) {
            self.advance();
            Some(self.parse_block_stmt()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_block: unwrap_block(then_block),
            else_block: else_block.map(unwrap_block),
        })
    }

    fn parse_paywall(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // consume 'paywall'
        self.expect(Token::LeftParen)?;
        
        // We expect a simple integer literal inside
        let amount = match self.current() {
            Some(Token::IntegerLiteral(n)) => *n as i64,
            _ => return Err(ParseError::UnexpectedToken),
        };
        self.advance(); // consume the number
        
        self.expect(Token::RightParen)?;
        self.expect(Token::SemiColon)?;
        
        Ok(Stmt::Paywall(amount))
    }

    // ----------------- expressions -----------------

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;

        while matches!(self.current(), Some(Token::GreaterThan | Token::LessThan)) {
            let op = match self.current().unwrap() {
                Token::GreaterThan => BinOp::GreaterThan,
                Token::LessThan => BinOp::LessThan,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_addition()?;

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        while matches!(self.current(), Some(Token::Plus | Token::Minus)) {
            let op = match self.current().unwrap() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_primary()?;

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Some(Token::IntegerLiteral(n)) => {
                let v = *n as i32; // i64 → i32
                self.advance();
                Ok(Expr::IntegerLiteral(v))
            }
            Some(Token::StringLiteral(s)) => {
                let v = s.clone();
                self.advance();
                Ok(Expr::StringLiteral(v))
            }
            Some(Token::Identifier(id)) => {
                let v = id.clone();
                self.advance();
                Ok(Expr::Identifier(v))
            }
            Some(Token::Maybe) => {
                self.advance();
                Ok(Expr::Maybe)
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken),
        }
    }
}

// ----------------- helpers -----------------

fn unwrap_block(stmt: Stmt) -> Vec<Stmt> {
    if let Stmt::Block(v) = stmt {
        v
    } else {
        unreachable!()
    }
}
