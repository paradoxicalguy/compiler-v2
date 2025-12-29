use std::collections::HashMap;
use crate::parsing::ast::{Expr, Stmt, BinOp};

pub struct Codegen {
    out: String,
    vars: HashMap<String, usize>, 
    stack_offset: usize,
    label_counter: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            out: String::new(),
            vars: HashMap::new(),
            stack_offset: 0,
            label_counter: 0,
        }
    }

    pub fn generate(mut self, stmts: &[Stmt]) -> String {
        // 1. DATA SECTION 
        let mut out = String::from("\t.data\n");
        out.push_str("fmt_int: .asciz \"%d\\n\"\n");
        out.push_str("fmt_str: .asciz \"%s\\n\"\n");
        
        // PAYWALL STRINGS
        out.push_str("fmt_scan: .asciz \"%s\"\n");
        out.push_str("msg_pay: .asciz \"free trial over pew pew, type 'haha' to continue: \"\n");
        out.push_str("secret:  .asciz \"haha\"\n");

        // 2. TEXT SECTION
        out.push_str("\n\t.text\n");
        out.push_str("\t.global main\n");
        out.push_str("main:\n");

        // Prologue
        out.push_str("\tstp x29, x30, [sp, #-16]!\n");
        out.push_str("\tmov x29, sp\n");
        out.push_str("\tsub sp, sp, #512\n");

        // Generate statements (populates self.out)
        for stmt in stmts {
            self.gen_stmt(stmt);
        }

        // Epilogue
        self.emit("\tadd sp, sp, #512");
        self.emit("\tldp x29, x30, [sp], #16");
        self.emit("\tmov x0, #0"); 
        self.emit("\tret");

        out + &self.out 
    }

    // --- STATEMENT GENERATION ---

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { name, value } => {
                let r = self.gen_expr(value);
                let offset = if let Some(&off) = self.vars.get(name) {
                    off
                } else {
                    let off = self.stack_offset;
                    self.vars.insert(name.clone(), off);
                    self.stack_offset += 8; 
                    off
                };
                self.emit(format!("\tstr {}, [sp, #{}]", r, offset));
            }

            Stmt::Print(expr) => {
                let r = self.gen_expr(expr);
                self.emit("\tadrp x0, fmt_int");
                self.emit("\tadd  x0, x0, :lo12:fmt_int");
                self.emit(format!("\tmov x1, {}", r));
                self.emit("\tbl printf");
            }

            Stmt::Block(stmts) => {
                for s in stmts { self.gen_stmt(s); }
            }

            Stmt::If { condition, then_block, else_block } => {
                let cond_reg = self.gen_expr(condition);
                let label_else = self.label("else");
                let label_end = self.label("endif");

                self.emit(format!("\tcmp {}, #0", cond_reg));
                self.emit(format!("\tbeq {}", label_else));

                for s in then_block { self.gen_stmt(s); }
                self.emit(format!("\tb {}", label_end));

                self.emit(format!("{}:", label_else));
                if let Some(block) = else_block {
                    for s in block { self.gen_stmt(s); }
                }
                self.emit(format!("{}:", label_end));
            }

            Stmt::ExprStmt(expr) => {
                self.gen_expr(expr);
            }

            // --- PAYWALL ---
            Stmt::Paywall(_) => {
                self.emit("\tadrp x0, msg_pay");
                self.emit("\tadd x0, x0, :lo12:msg_pay");
                self.emit("\tbl printf");

                self.emit("\tadrp x0, fmt_scan");
                self.emit("\tadd x0, x0, :lo12:fmt_scan");
                self.emit("\tadd x1, sp, #400"); // buffer at sp+400
                self.emit("\tbl scanf");

                self.emit("\tadd x0, sp, #400");
                self.emit("\tadrp x1, secret");
                self.emit("\tadd x1, x1, :lo12:secret");
                self.emit("\tbl strcmp");

                let label_paid = self.label("paid");
                self.emit("\tcmp x0, #0");
                self.emit(format!("\tbeq {}", label_paid));

                // Exit if wrong
                self.emit("\tmov x0, #1"); 
                self.emit("\tmov x8, #93");
                self.emit("\tsvc #0");

                self.emit(format!("{}:", label_paid));
            }
        }
    }

    // --- EXPRESSION GENERATION ---

    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntegerLiteral(n) => {
                let r = self.alloc_tmp();
                self.emit(format!("\tldr {}, ={}", r, n));
                r
            }
            Expr::Identifier(name) => {
                let r = self.alloc_tmp();
                let offset = self.vars.get(name).copied().unwrap_or(0);
                self.emit(format!("\tldr {}, [sp, #{}]", r, offset));
                r
            }
            Expr::Binary { left, op, right } => {
                let r1 = self.gen_expr(left);
                let r2 = self.gen_expr(right);
                let dest = self.alloc_tmp();

                match op {
                    BinOp::Add => self.emit(format!("\tadd {}, {}, {}", dest, r1, r2)),
                    BinOp::Sub => self.emit(format!("\tsub {}, {}, {}", dest, r1, r2)),
                    BinOp::GreaterThan => {
                        self.emit(format!("\tcmp {}, {}", r1, r2));
                        self.emit(format!("\tcset {}, gt", dest));
                    }
                    BinOp::LessThan => {
                        self.emit(format!("\tcmp {}, {}", r1, r2));
                        self.emit(format!("\tcset {}, lt", dest));
                    }
                }
                dest
            }
            Expr::Maybe => {
                let r = self.alloc_tmp();
                self.emit("\tbl rand");
                self.emit(format!("\tand {}, x0, #1", r));
                r
            }
            _ => { 
                let r = self.alloc_tmp(); 
                self.emit(format!("\tmov {}, #0", r)); 
                r 
            }
        }
    }

    fn alloc_tmp(&self) -> String {
        "x14".to_string() 
    }

    fn label(&mut self, prefix: &str) -> String {
        let l = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        l
    }

    fn emit(&mut self, asm: impl Into<String>) {
        self.out.push_str(&asm.into());
        self.out.push('\n');
    }
}