use std::collections::HashMap;
use crate::ast::{Expr, Stmt, BinOp};

// types supported by the language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    String,
    Bool,
    Unknown,
}

// semantic errors produced during analysis
#[derive(Debug, Clone)]
pub enum SemanticError {
    UndeclaredVariable(String),
    Redeclaration(String),
    InvalidAssignmentTarget(String),
    TypeMismatch {
        expected: Type,
        found: Type,
        context: String,
    },
}

// the semantic analyzer holds scopes (a stack of symbol tables) and collected errors
pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, Type>>,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scopes: vec![HashMap::new()], // global scope
            errors: Vec::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // lookup variable from innermost to outermost scope
    fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    // declare a variable in the innermost scope
    fn declare_variable(&mut self, name: &str, var_type: Type) -> bool {
        let current_scope = self.scopes.last_mut().expect("always at least one scope");
        if current_scope.contains_key(name) {
            return false;
        }
        current_scope.insert(name.to_string(), var_type);
        true
    }

    fn add_error(&mut self, err: SemanticError) {
        self.errors.push(err);
    }

    // analyze a list of statements (program), eturns Ok if no semantic errors; otherwise returns the errors.
    pub fn analyze(&mut self, statements: &[Stmt]) -> Result<(), Vec<SemanticError>> {
        for stmt in statements {
            self.check_statement(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    // single entry for statements
    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { name, value } => {
                self.check_var_declaration(name, value);
            }
            Stmt::Print(expr) => {
                // validate expression (type returned is ignored for print)
                let _ = self.check_expression(expr);
            }
            Stmt::If { condition, then_block, else_block } => {
                self.check_if_statement(condition, then_block, else_block);
            }
        }
    }

    // handle `int name = value;` declarations
    // note: parser only allows `int` declarations, so we treat the declared type as `Int`
    fn check_var_declaration(&mut self, name: &str, value: &Expr) {
        // if already declared in current scope -> redeclaration error
        let current_scope = self.scopes.last().expect("at least one scope");
        if current_scope.contains_key(name) {
            self.add_error(SemanticError::Redeclaration(name.to_string()));
            return;
        }

        // evaluate initializer type
        let value_type = self.check_expression(value);

        // declared type is Int (because your parser uses `int`)
        let declared_type = Type::Int;

        // if initializer's type is known and doesn't match declared type -> type mismatch
        if value_type != Type::Unknown && value_type != declared_type {
            self.add_error(SemanticError::TypeMismatch {
                expected: declared_type.clone(),
                found: value_type.clone(),
                context: format!("initializer for '{}' must be {:?}", name, declared_type),
            });
        }

        // insert the variable into current scope with the declared type (even if initializer mismatched)
        // (alternatively you could skip insertion on mismatch; this choice keeps semantics stable)
        let current_scope_mut = self.scopes.last_mut().expect("at least one scope");
        current_scope_mut.insert(name.to_string(), declared_type);
    }

    // check an if statement: validate condition and check then/else blocks with their own scope
    fn check_if_statement(&mut self, condition: &Expr, then_block: &[Stmt], else_block: &Option<Vec<Stmt>>) {
        let cond_type = self.check_expression(condition);
        // condition must be boolean; we added Type::Bool
        if cond_type != Type::Bool && cond_type != Type::Unknown {
            self.add_error(SemanticError::TypeMismatch {
                expected: Type::Bool,
                found: cond_type,
                context: "if condition must be boolean".to_string(),
            });
        }

        // then block runs in its own nested scope
        self.enter_scope();
        for stmt in then_block {
            self.check_statement(stmt);
        }
        self.exit_scope();

        // else block if present
        if let Some(else_stmts) = else_block {
            self.enter_scope();
            for stmt in else_stmts {
                self.check_statement(stmt);
            }
            self.exit_scope();
        }
    }

    // evaluate an expression and return its inferred Type. Add errors when rules are violated.
    fn check_expression(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntegerLiteral(_) => Type::Int,
            Expr::StringLiteral(_) => Type::String,
            Expr::Identifier(name) => self.check_identifier(name),
            Expr::Binary { left, op, right } => self.check_binary_expression(left, op, right),
            Expr::Assign { name, value } => self.check_assignment(name, value),
        }
    }

    // identifier usage: return type if found, otherwise add undeclared error and return Unknown.
    fn check_identifier(&mut self, name: &str) -> Type {
        match self.lookup_variable(name) {
            Some(t) => t,
            None => {
                self.add_error(SemanticError::UndeclaredVariable(name.to_string()));
                Type::Unknown
            }
        }
    }

    /// binary expression rules:
    /// - add: int+int -> Int ; string+string -> String
    /// - sub: int-int -> Int
    /// - greater/less: int op int -> bool
    fn check_binary_expression(&mut self, left: &Expr, op: &BinOp, right: &Expr) -> Type {
        let left_type = self.check_expression(left);
        let right_type = self.check_expression(right);

        match op {
            BinOp::Add => {
                if left_type == Type::Int && right_type == Type::Int {
                    Type::Int
                } else if left_type == Type::String && right_type == Type::String {
                    Type::String
                } else {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: left_type.clone(), // not perfect but informative
                        found: right_type.clone(),
                        context: "addition requires both sides to have the same type (Int+Int or String+String)".to_string(),
                    });
                    Type::Unknown
                }
            }

            BinOp::Sub => {
                if left_type == Type::Int && right_type == Type::Int {
                    Type::Int
                } else {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: Type::Int,
                        found: if left_type != Type::Int { left_type.clone() } else { right_type.clone() },
                        context: "subtraction requires both operands to be Int".to_string(),
                    });
                    Type::Unknown
                }
            }

            BinOp::GreaterThan | BinOp::LessThan => {
                if left_type == Type::Int && right_type == Type::Int {
                    Type::Bool
                } else {
                    self.add_error(SemanticError::TypeMismatch {
                        expected: Type::Int,
                        found: if left_type != Type::Int { left_type.clone() } else { right_type.clone() },
                        context: "comparison requires both sides to be Int".to_string(),
                    });
                    Type::Unknown
                }
            }
        }
    }

    fn check_assignment(&mut self, name: &str, value: &Expr) -> Type {
        // variable must exist
        let var_type = match self.lookup_variable(name) {
            Some(t) => t,
            None => {
                self.add_error(SemanticError::UndeclaredVariable(name.to_string()));
                return Type::Unknown;
            }
        };

        let value_type = self.check_expression(value);

        // if value's type known and mismatches variable type -> error
        if value_type != Type::Unknown && var_type != value_type {
            self.add_error(SemanticError::TypeMismatch {
                expected: var_type.clone(),
                found: value_type.clone(),
                context: format!("cannot assign value of type {:?} to variable '{}' of type {:?}", value_type, name, var_type),
            });
        }

        // assignment expression's type is the value's type (common convention)
        value_type
    }
}
