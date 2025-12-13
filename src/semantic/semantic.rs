use std::collections::HashMap;
use crate::parsing::ast::{Expr, Stmt, BinOp};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    String,
    Bool,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndeclaredVariable(String),
    Redeclaration(String),
    TypeMismatch {
        expected: Type,
        found: Type,
        context: String,
    },
}

pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, Type>>,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            errors: Vec::new(),
        }
    }

    // ---------- scopes ----------

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&mut self) -> &mut HashMap<String, Type> {
        self.scopes.last_mut().unwrap()
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn error(&mut self, err: SemanticError) {
        self.errors.push(err);
    }

    // ---------- entry ----------

    pub fn analyze(&mut self, stmts: &[Stmt]) -> Result<(), Vec<SemanticError>> {
        for s in stmts {
            self.check_stmt(s);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    // ---------- statements ----------

    fn check_stmt(&mut self, stmt: &Stmt) {
    match stmt {
        Stmt::Block(stmts) => {
            self.enter_scope();
            for s in stmts {
                self.check_stmt(s);
            }
            self.exit_scope();
        }

        Stmt::VarDeclaration { name, value } => {
            self.check_var_decl(name, value);
        }

        Stmt::Print(expr) => {
            self.check_expr(expr);
        }

        Stmt::If { condition, then_block, else_block } => {
            self.check_if(condition, then_block, else_block);
        }

        // ✅ THIS FIX
        Stmt::ExprStmt(expr) => {
            self.check_expr(expr);
        }
    }
}


    fn check_var_decl(&mut self, name: &str, value: &Expr) {
        if self.current_scope().contains_key(name) {
            self.error(SemanticError::Redeclaration(name.to_string()));
            return;
        }

        let value_type = self.check_expr(value);

        if value_type != Type::Int && value_type != Type::Unknown {
            self.error(SemanticError::TypeMismatch {
                expected: Type::Int,
                found: value_type,
                context: format!("initializer for '{}' must be Int", name),
            });
        }

        self.current_scope().insert(name.to_string(), Type::Int);
    }

    fn check_if(&mut self, cond: &Expr, then_block: &[Stmt], else_block: &Option<Vec<Stmt>>) {
        let cond_type = self.check_expr(cond);

        if cond_type != Type::Bool && cond_type != Type::Unknown {
            self.error(SemanticError::TypeMismatch {
                expected: Type::Bool,
                found: cond_type,
                context: "if condition must be boolean".to_string(),
            });
        }

        self.enter_scope();
        for s in then_block {
            self.check_stmt(s);
        }
        self.exit_scope();

        if let Some(stmts) = else_block {
            self.enter_scope();
            for s in stmts {
                self.check_stmt(s);
            }
            self.exit_scope();
        }
    }

    // ---------- expressions ----------

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntegerLiteral(_) => Type::Int,
            Expr::StringLiteral(_) => Type::String,
            Expr::BooleanLiteral(_) => Type::Bool,


            Expr::Identifier(name) => {
                self.lookup(name).unwrap_or_else(|| {
                    self.error(SemanticError::UndeclaredVariable(name.clone()));
                    Type::Unknown
                })
            }

            Expr::Assign { name, value } => {
                let var_type = self.lookup(name).unwrap_or_else(|| {
                    self.error(SemanticError::UndeclaredVariable(name.clone()));
                    Type::Unknown
                });

                let value_type = self.check_expr(value);

                if var_type != Type::Unknown && value_type != Type::Unknown && var_type != value_type {
                    self.error(SemanticError::TypeMismatch {
                        expected: var_type.clone(),
                        found: value_type.clone(),
                        context: format!("cannot assign to '{}'", name),
                    });
                }

                var_type
            }

            Expr::Binary { left, op, right } => self.check_binary(left, op, right),
        }
    }

    fn check_binary(&mut self, left: &Expr, op: &BinOp, right: &Expr) -> Type {
        let lt = self.check_expr(left);
        let rt = self.check_expr(right);

        match op {
            BinOp::Add => {
                if lt == Type::Int && rt == Type::Int {
                    Type::Int
                } else if lt == Type::String && rt == Type::String {
                    Type::String
                } else {
                    self.error(SemanticError::TypeMismatch {
                        expected: lt,
                        found: rt,
                        context: "invalid '+' operands".to_string(),
                    });
                    Type::Unknown
                }
            }

            BinOp::Sub => {
                if lt == Type::Int && rt == Type::Int {
                    Type::Int
                } else {
                    self.error(SemanticError::TypeMismatch {
                        expected: Type::Int,
                        found: if lt != Type::Int { lt } else { rt },
                        context: "subtraction requires Int".to_string(),
                    });
                    Type::Unknown
                }
            }

            BinOp::GreaterThan | BinOp::LessThan => {
                if lt == Type::Int && rt == Type::Int {
                    Type::Bool
                } else {
                    self.error(SemanticError::TypeMismatch {
                        expected: Type::Int,
                        found: if lt != Type::Int { lt } else { rt },
                        context: "comparison requires Int".to_string(),
                    });
                    Type::Unknown
                }
            }
        }
    }
}
