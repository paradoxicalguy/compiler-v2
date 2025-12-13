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
    used_vars: HashSet<String>,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            used_vars: HashSet::new(),
        }
    }

    // -------- ENTRY --------

    pub fn optimize(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut current = stmts;

        for _ in 0..10 {
            self.constants.clear();
            self.used_vars.clear();

            self.collect_used_vars(&current);
            let optimized = self.optimize_stmts(current);
            let cleaned = self.dead_code_elimination(optimized.clone());

            if optimized == current {
                break;
            }

            current = cleaned;
        }

        current
    }

    // -------- STATEMENTS --------

    fn optimize_stmts(&mut self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        stmts.into_iter().flat_map(|s| self.optimize_stmt(s)).collect()
    }

    fn optimize_stmt(&mut self, stmt: Stmt) -> Vec<Stmt> {
        match stmt {
            Stmt::VarDeclaration { name, value } => {
                let value = self.optimize_expr(value);

                if let Some(c) = self.eval_const(&value) {
                    self.constants.insert(name.clone(), c);
                } else {
                    self.constants.remove(&name);
                }

                vec![Stmt::VarDeclaration { name, value }]
            }

            Stmt::Print(expr) => {
                vec![Stmt::Print(self.optimize_expr(expr))]
            }

            Stmt::Block(stmts) => {
                vec![Stmt::Block(self.optimize_stmts(stmts))]
            }

            Stmt::If { condition, then_block, else_block } => {
                self.optimize_if(condition, then_block, else_block)
            }
        }
    }

    fn optimize_if(
        &mut self,
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    ) -> Vec<Stmt> {
        let cond = self.optimize_expr(condition);

        if let Some(ConstValue::Bool(b)) = self.eval_const(&cond) {
            if b {
                return self.optimize_stmts(then_block);
            } else {
                return else_block.map(self.optimize_stmts).unwrap_or_default();
            }
        }

        vec![Stmt::If {
            condition: cond,
            then_block: self.optimize_stmts(then_block),
            else_block: else_block.map(|b| self.optimize_stmts(b)),
        }]
    }

    // -------- EXPRESSIONS --------

    fn optimize_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::Identifier(name) => {
                if let Some(c) = self.constants.get(&name) {
                    match c {
                        ConstValue::Int(n) => Expr::IntegerLiteral(*n),
                        ConstValue::String(s) => Expr::StringLiteral(s.clone()),
                        ConstValue::Bool(b) => Expr::BooleanLiteral(*b),
                    }
                } else {
                    Expr::Identifier(name)
                }
            }

            Expr::Binary { left, op, right } => {
                self.optimize_binary(*left, op, *right)
            }

            Expr::Assign { name, value } => {
                let v = self.optimize_expr(*value);
                self.constants.remove(&name);
                Expr::Assign { name, value: Box::new(v) }
            }

            _ => expr,
        }
    }

    fn optimize_binary(&mut self, left: Expr, op: BinOp, right: Expr) -> Expr {
        let l = self.optimize_expr(left);
        let r = self.optimize_expr(right);

        if let (Some(lc), Some(rc)) = (self.eval_const(&l), self.eval_const(&r)) {
            if let Some(result) = self.fold(lc, &op, rc) {
                return result;
            }
        }

        match (&op, &l, &r) {
            (BinOp::Add, Expr::IntegerLiteral(0), _) => r,
            (BinOp::Add, _, Expr::IntegerLiteral(0)) => l,
            (BinOp::Sub, _, Expr::IntegerLiteral(0)) => l,
            _ => Expr::Binary {
                left: Box::new(l),
                op,
                right: Box::new(r),
            },
        }
    }

    // -------- CONSTANT FOLDING --------

    fn eval_const(&self, expr: &Expr) -> Option<ConstValue> {
        match expr {
            Expr::IntegerLiteral(n) => Some(ConstValue::Int(*n)),
            Expr::StringLiteral(s) => Some(ConstValue::String(s.clone())),
            Expr::BooleanLiteral(b) => Some(ConstValue::Bool(*b)),
            _ => None,
        }
    }

    fn fold(&self, l: ConstValue, op: &BinOp, r: ConstValue) -> Option<Expr> {
        match (l, op, r) {
            (ConstValue::Int(a), BinOp::Add, ConstValue::Int(b)) =>
                Some(Expr::IntegerLiteral(a + b)),

            (ConstValue::Int(a), BinOp::Sub, ConstValue::Int(b)) =>
                Some(Expr::IntegerLiteral(a - b)),

            (ConstValue::Int(a), BinOp::GreaterThan, ConstValue::Int(b)) =>
                Some(Expr::BooleanLiteral(a > b)),

            (ConstValue::Int(a), BinOp::LessThan, ConstValue::Int(b)) =>
                Some(Expr::BooleanLiteral(a < b)),

            (ConstValue::String(a), BinOp::Add, ConstValue::String(b)) =>
                Some(Expr::StringLiteral(format!("{}{}", a, b))),

                println!("FOLDING: {:?} {:?} {:?}", l, op, r);
            _ => None,
        }
    }

    // -------- DEAD CODE --------

    fn collect_used_vars(&mut self, stmts: &[Stmt]) {
        for s in stmts {
            self.collect_stmt(s);
        }
    }

    fn collect_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { value, .. } => self.collect_expr(value),
            Stmt::Print(e) => self.collect_expr(e),
            Stmt::If { condition, then_block, else_block } => {
                self.collect_expr(condition);
                then_block.iter().for_each(|s| self.collect_stmt(s));
                if let Some(b) = else_block {
                    b.iter().for_each(|s| self.collect_stmt(s));
                }
            }
            Stmt::Block(stmts) => stmts.iter().for_each(|s| self.collect_stmt(s)),
        }
    }

    fn collect_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(n) => {
                self.used_vars.insert(n.clone());
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expr(left);
                self.collect_expr(right);
            }
            Expr::Assign { name, value } => {
                self.used_vars.insert(name.clone());
                self.collect_expr(value);
            }
            _ => {}
        }
    }

    fn dead_code_elimination(&self, stmts: Vec<Stmt>) -> Vec<Stmt> {
        stmts.into_iter()
            .filter(|s| match s {
                Stmt::VarDeclaration { name, .. } =>
                    self.used_vars.contains(name),
                _ => true,
            })
            .collect()
    }
}
