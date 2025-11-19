use std::collections::{HashMap, HashSet};
use crate::ast::{Expr, Stmt, BinOp};

#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i32),      
    String(String),
    Bool(bool),    
}


pub struct Optimizer {
    constants: HashMap<String, ConstValue>,
    used_variables: HashSet<String>,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {
            constants: HashMap::new(),
            used_variables: HashSet::new(),
        }
    }

    pub fn new() -> Self {
        Optimizer {
            constants: Hashmap::new(),
            used_variables: HashSet::new(),
        }
    }

    pub fn optimize(&mut self, statements: Vec<Stmt>) -> Vec<Stmt> {
        let mut current = statements;

        let mut iteration = 0;
        loop {
            iteration += 1;

            let before = format! ("{:?}", current);

            self.used_variables.clear();
            self.collect_used_variables(&current);

            current = self.optimize_statements(current);
            current = self.eliminate_dead_code(current);
            let after = format!("{:?}", current);

            if before == after {
                break;
            }

            if iteration > 100 {
                eprintln!("warning: optimization didnt converge after 100 iterations");
                break;
            }
        }
        current
    }

    pub fn optimize_statement(&mut self, stmt: &Stmt) -> Option<Stmt> {
        match Stmt {
            Stmt::VarDeclaration {name, value} => {
                let optimized_value = self.optimize_expression(value);

                if let Somme(const_val) = self.try_evaluate_to_const(&optimized_value) {
                    self.constants.insert(name.clone(), const_val);
                } else {
                    self.constants.remove(&name);
                }
                Some(Stmt::VarDeclaration {
                    name, 
                    value: optimized_value,
                })
            }

            Stmt::Print(expr) => {
                let optimized_expr = self.optimize_expression(expr);
                Some(Stmt::Print(optimized_expr))
            }

            Stmt::If {condition, then_block, else_block } => {
                self.optimize_if_statement(condition, then_block, else_block)
            }
        }
    }
    
    fn optimize_if_statement(&mut self, 
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>
    ) -> Option<Stmt> {
        let optimized_condition = self.optimize_expression(condition);

        if let Some(const_val) = self.try_evaluate_to_const(&optimized_condition) {
            match const_val {
                ConstValue::Bool(true) => {
                    let optimized_then = self.optimize_statements(then_block);

                    return Seome(Stmt::If {
                        condition: optimized_condition,
                        then_block: optimized_then,
                        else_block: None,
                    });
                }

                ConstValue::Bool(false) => {
                    if let Some(else_stmts) = else_block {
                        let optimized_else = self.optimize_statements(else_stmts);
                        return Some(Stmt::If {
                            condition: optimized_condition,
                            then_block: vec![],
                            else_block: Some(optimized_else),
                        });
                    } else {
                        return None;
                    }
                }
                _ => {

                }
            }
        }
        let optimized_then = self.optimize_statements(then_block);
        let optimized_else = else_block.map(|stmts| self.optimize_statements(stmts));

        Some(Stmt::If {
            condition: optimized_condition,
            then_block: optimized_then,
            else_block: optimized_else
        })
    }

    fn optimize_expression(&self, expr: Expr) -> Expr {
        match expr {
            Expr::IntegerLiteral(_) | Expr::StringLiteral(_) => expr,

            Expr::Identifier(name) => {
                if let Some(const_val) = self.constants.get(&name) {
                    match const_val {
                        ConstValue::Int(n) => Expr::IntegerLiteral(*n),
                        ConstValue::String(s) => Expr::StringLiteral(s.clone()),
                        ConstValue::Bool(_b) => {
                            Expr::Identifier(name)
                        }
                    }
                } else {
                    Expr::Identifier(name)
                }
            }
            Expr::Binary {left, op, right} => {
                self.optimize_binary_expression(*left, op, *right)
            }
            Expr::Assign {name, value} => {
                let optimized_value = self.optimize_expression(*value);
                Expr::Assign {
                    name, 
                    value: Box::new(optimized_value),
                }
            }
        }
    }

    fn optimize_binary_expression (&self, left:  Expr, op: BinOp, right: Expr) -> Expr {
        let opt_left = self.optimize_expression(left);
        let opt_right = self.optimize_expression(right);

        let left_const = self.try_evaluate_to_const(&opt_left);
        let right_const = self.try_evaluate_to_const(&opt_right);

        if let (Some(l), Some(r)) = (&left_const, &right_const) {
            if let Some(result) = self.fold_binary_operation(l, &op, r) {
                return result;
            }
        }

        match op {
            BinOp::Add => {
                if let Some(ConstValue::Int(0)) = right_const {
                    return opt_left;
                }
                if let Some(ConstValue::Int(0)) = left_const {
                    return opt_right;
                }
            }

            BinOp::Sub => {
                if let Some(ConstValue::Int(0)) = right_const {
                    return opt_left;
                }
                if let (Expr::Identifier(l), Expr::Identifier(r)) = (&opt_left, &opt_right) {
                    if l == r {
                        return Expr::IntegerLiteral(0);
                    }
                }
            }
            _ => {}
        }

        Expr::Binary {
            left: Box::new(opt_left),
            op,
            right: Box::new(opt_right),
        }
    }

    fn try_evaluate_to_const(&self, expr: &Expr) -> Option<ConstValue> {
        match expr {
            Expr::IntegerLiteral(n) => Some(ConstValue::Int(*n)),
            Expr::StringLiteral(s) => Some(ConstValue::String(*s)),

            Expr::Identifier(name) => {
                self.constants.get(name).cloned()
            }
            Expr::Binary {left, op, right} => {
                let left_val = self.try_evaluate_to_const(left)?;
                let right_val = self.try_evaluate_to_const(right)?;

                self.fold_binary_operation(&left_val, op, &right_val)
                    .and_then(|expr| self.try_evaluate_to_const(&expr))
            }

            Expr::Assign{..} => {
                None
            }
        }
    }

    fn fold_binary_operation (
        &self,
        left: &ConstValue,
        op: &BinOp,
        right: &ConstValue,
    ) -> Option<Expr> {
        match(left, op, right) {
                (ConstValue::Int(l), BinOp::Add, ConstValue::Int(r)) => {
                Some(Expr::IntegerLiteral(l + r))
            }
                (ConstValue::Int(l), BinOp::Sub, ConstValue::Int(r)) => {
                    Some(Expr::IntegerLiteral(l - r))
            }
            (ConstValue::Int(l), BinOp::GreaterThan, ConstValue::Int(r)) => {
                let result = if l > r { 1 } else { 0 };
                Some(Expr::BooleanLiteral(result))
            }
            (ConstValue::Int(l), BinOp::LessThan, ConstValue::Int(r)) => {
                let result = if l < r { 1 } else { 0 };
                Some(Expr::BooleanLiteral(result))
            }

            (ConstValue::String(l), BinOp::Add, ConstValue::String(r)) => {
                Some(Expr::StringLiteral(format!("{}{}", l, r)))
            }

            _ => None
        }
    }
    fn collect_used_variables(&mut self, statements: &[Stmt]) {

        for stmt in statements {

            self.collect_used_in_statement(stmt);

        }

    }
    fn collect_used_in_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { name: _, value } => {
                self.collect_used_in_expression(value);
            }
            Stmt::Print(expr) => {
                self.collect_used_in_expression(expr);
            }

            Stmt::If { condition, then_block, else_block } => {
                self.collect_used_in_expression(condition);
                for stmt in then_block {
                    self.collect_used_in_statement(stmt);
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.collect_used_in_statement(stmt);
                    }
                }
            }
        }
    }
    fn collect_used_in_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::IntegerLiteral(_) | Expr::StringLiteral(_) => {
            }
            Expr::Identifier(name) => {
               self.used_variables.insert(name.clone());
            }
            Expr::Binary { left, right, .. } => {
                self.collect_used_in_expression(left);
                self.collect_used_in_expression(right);
            }

            Expr::Assign { name, value } => {
                self.used_variables.insert(name.clone());
                self.collect_used_in_expression(value);
            }
        }
    }
    fn eliminate_dead_code(&self, statements: Vec<Stmt>) -> Vec<Stmt> {
        statements
            .into_iter()
            .filter_map(|stmt| match stmt {
                Stmt::VarDeclaration { ref name, .. } => {
                    if self.used_variables.contains(name) {
                        Some(stmt) // Keep it
                    } else {
                        None 
                    }
                }
                _ => Some(stmt),
            })
            .collect()
    }
}
