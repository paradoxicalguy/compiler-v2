use std::collections::HashMap;
use crate::parsing::ast::{Expr, Stmt, BinOp};

pub struct Codegen {
    data_strings: Vec<(String, String)>,
    vars: HashMap<String, i32>,
    next_offset: i32,
    instrs: Vec<String>,
    label_id: usize,
    temps: Vec<&'static str>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            data_strings: Vec::new(),
            vars: HashMap::new(),
            next_offset: 0,
            instrs: Vec::new(),
            label_id: 0,
            temps: vec!["x9","x10","x11","x12","x13","x14","x15"],
        }
    }

    // ================== ENTRY ==================

    pub fn generate(mut self, stmts: &[Stmt]) -> String {
        self.emit_prologue();

        for s in stmts {
            self.gen_stmt(s);
        }

        self.emit_epilogue();

        let mut out = String::new();

        // ---------- DATA ----------
        out.push_str("\t.data\n");
        out.push_str("fmt_int: .asciz \"%d\\n\"\n");
        out.push_str("fmt_str: .asciz \"%s\\n\"\n");

        for (lbl, txt) in &self.data_strings {
            out.push_str(&format!("{}: .asciz \"{}\"\n", lbl, txt));
        }

        // ---------- TEXT ----------
        out.push_str("\n\t.text\n");
        out.push_str("\t.global main\n");

        for i in &self.instrs {
            out.push_str(i);
            out.push('\n');
        }

        out
    }

    // ================== HELPERS ==================

    fn emit(&mut self, s: impl Into<String>) {
        self.instrs.push(s.into());
    }

    fn label(&mut self, base: &str) -> String {
        let l = format!("{}_{}", base, self.label_id);
        self.label_id += 1;
        l
    }

    fn intern(&mut self, s: &str) -> String {
        let lbl = format!(".LC{}", self.data_strings.len());
        self.data_strings.push((lbl.clone(), s.to_string()));
        lbl
    }

    fn alloc_var(&mut self, name: &str) -> i32 {
        *self.vars.entry(name.to_string()).or_insert_with(|| {
            let off = self.next_offset;
            self.next_offset += 8;
            off
        })
    }

    fn alloc_tmp(&mut self) -> &'static str {
        self.temps.pop().expect("out of registers")
    }

    fn free_tmp(&mut self, r: &'static str) {
        self.temps.push(r);
    }

    // ================== PROLOGUE ==================

    fn emit_prologue(&mut self) {
        self.emit("main:");
        self.emit("\tstp x29, x30, [sp, #-16]!");
        self.emit("\tmov x29, sp");
        self.emit("\tsub sp, sp, #512");
    }

    fn emit_epilogue(&mut self) {
        self.emit("\tadd sp, sp, #512");
        self.emit("\tldp x29, x30, [sp], #16");
        self.emit("\tmov x0, #0");
        self.emit("\tret");
    }

    // ================== STATEMENTS ==================

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.gen_stmt(s);
                }
            }

            Stmt::VarDeclaration { name, value } => {
                let off = self.alloc_var(name);
                let r = self.gen_expr(value);
                self.emit(format!("\tstr {}, [sp, #{}]", r, off));
                self.free_tmp(r);
            }

            Stmt::Print(expr) => match expr {
                Expr::StringLiteral(s) => {
                    let lbl = self.intern(s);
                    self.emit("\tadrp x0, fmt_str");
                    self.emit("\tadd  x0, x0, :lo12:fmt_str");
                    self.emit(format!("\tadrp x1, {}", lbl));
                    self.emit(format!("\tadd  x1, x1, :lo12:{}", lbl));
                    self.emit("\tbl printf");
                }
                _ => {
                    let r = self.gen_expr(expr);
                    self.emit("\tadrp x0, fmt_int");
                    self.emit("\tadd  x0, x0, :lo12:fmt_int");
                    self.emit(format!("\tmov x1, {}", r));
                    self.emit("\tbl printf");
                    self.free_tmp(r);
                }
            },

            Stmt::If { condition, then_block, else_block } => {
                let r = self.gen_expr(condition);
                let else_l = self.label("else");
                let end_l = self.label("endif");

                self.emit(format!("\tcmp {}, #0", r));
                self.emit(format!("\tbeq {}", else_l));

                for s in then_block {
                    self.gen_stmt(s);
                }

                self.emit(format!("\tb {}", end_l));
                self.emit(format!("{}:", else_l));

                if let Some(stmts) = else_block {
                    for s in stmts {
                        self.gen_stmt(s);
                    }
                }

                self.emit(format!("{}:", end_l));
                self.free_tmp(r);
            }

            Stmt::ExprStmt(expr) => {
                let r = self.gen_expr(expr);
                self.free_tmp(r);
            }
        }
    }

    // ================== EXPRESSIONS ==================

    fn gen_expr(&mut self, expr: &Expr) -> &'static str {
        match expr {
            Expr::IntegerLiteral(n) => {
                let r = self.alloc_tmp();
                self.emit(format!("\tmov {}, #{}", r, n));
                r
            }

            Expr::StringLiteral(s) => {
                let lbl = self.intern(s);
                let r = self.alloc_tmp();
                self.emit(format!("\tadrp {}, {}", r, lbl));
                self.emit(format!("\tadd  {}, {}, :lo12:{}", r, r, lbl));
                r
            }
            Expr::BooleanLiteral(b) => {
                let r = self.alloc_tmp();
                self.emit(format!(
                    "\tmov {}, #{}",
                    r,
                    if *b { 1 } else { 0 }
                ));
                r
            }


            Expr::Identifier(name) => {
                let r = self.alloc_tmp();
                let off = self.vars.get(name).copied().unwrap_or(0);
                self.emit(format!("\tldr {}, [sp, #{}]", r, off));
                r
            }

            Expr::Assign { name, value } => {
                let r = self.gen_expr(value);
                let off = self.alloc_var(name);
                self.emit(format!("\tstr {}, [sp, #{}]", r, off));
                r
            }

            Expr::Binary { left, op, right } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);

                match op {
                    BinOp::Add => self.emit(format!("\tadd {}, {}, {}", l, l, r)),
                    BinOp::Sub => self.emit(format!("\tsub {}, {}, {}", l, l, r)),
                    BinOp::GreaterThan => {
                        self.emit(format!("\tcmp {}, {}", l, r));
                        self.emit(format!("\tcset {}, gt", l));
                    }
                    BinOp::LessThan => {
                        self.emit(format!("\tcmp {}, {}", l, r));
                        self.emit(format!("\tcset {}, lt", l));
                    }
                }

                self.free_tmp(r);
                l
            }
        }
    }
}
