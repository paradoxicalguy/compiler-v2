
use std::collections::HashMap;
use crate::ast::{Expr, Stmt, BinOp};

pub struct Codegen {
    data_strings: Vec<(String, String)>,            
    var_offsets: HashMap<String, i32>,              
    next_var_offset: i32,                           
    instrs: Vec<String>,                            
    label_counter: usize,                           
    temp_regs: Vec<&'static str>,                   
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            data_strings: Vec::new(),
            var_offsets: HashMap::new(),
            next_var_offset: 0,
            instrs: Vec::new(),
            label_counter: 0,
            temp_regs: vec!["x9","x10","x11","x12","x13","x14","x15"],
        }
    }

    pub fn generate(mut self, stmts: &[Stmt]) -> String {
        self.emit_prologue();
        for s in stmts {
            self.gen_stmt(s);
        }
        self.emit_epilogue();

        let mut data_section = Vec::new();
        data_section.push(r#"fmt_int: .asciz "%d\n""#.to_string());
        data_section.push(r#"fmt_str: .asciz "%s\n""#.to_string());
        for (label, text) in &self.data_strings {
            data_section.push(format!("{}: .asciz {}", label, text));
        }
        
        let mut out = String::new();
        out.push_str("\t.data\n");
        for line in data_section {
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("\n\t.text\n");
        out.push_str("\t.global main\n");
        
        for instr in &self.instrs {
            out.push_str(instr);
            out.push('\n');
        }
        out
    }

    fn emit(&mut self, line: impl Into<String>) {
        self.instrs.push(line.into());
    }

    fn fresh_label(&mut self, base: &str) -> String {
        let lbl = format!("{}_{}", base, self.label_counter);
        self.label_counter += 1;
        lbl
    }

    fn intern_string(&mut self, s: &str) -> String {
        let label = format!(".LC{}", self.data_strings.len());
        self.data_strings.push((label.clone(), s.to_string()));
        label
    }

    fn allocate_var(&mut self, name: &str) -> i32 {
        if let Some(off) = self.var_offsets.get(name) {
            return *off;
        }
        let off = self.next_var_offset;
        self.var_offsets.insert(name.to_string(), off);
        self.next_var_offset += 8;
        off
    }

    fn lookup_var(&self, name: &str) -> Option<i32> {
        self.var_offsets.get(name).copied()
    }
    fn alloc_tmp(&mut self) -> Option<&'static str> {
        self.temp_regs.pop()
    }

    fn free_tmp(&mut self, reg: &'static str) {
        self.temp_regs.push(reg);
    }

    fn emit_prologue(&mut self) {
        self.emit("\tmain:");
        self.emit("\tstp x29, x30, [sp, #-16]!    // save fp and lr, push 16 bytes");
        self.emit("\tmov x29, sp                  // set frame pointer");
        self.emit("\tsub sp, sp, #512            // reserve 512 bytes for locals (simple stack frame)");
    }

    fn emit_epilogue(&mut self) {
        self.emit("\tadd sp, sp, #512            // deallocate frame");
        self.emit("\tldp x29, x30, [sp], #16     // restore fp and lr");
        self.emit("\tmov x0, #0                  // return 0");
        self.emit("\tret");
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { name, value } => {
                let offset = self.allocate_var(name);
                let reg = self.gen_expr(value);

                self.emit(format!("\tstr {}, [sp, #{}]    // store '{}' into local slot", reg, offset, name));
                self.free_tmp(reg);
            }
            Stmt::Print(expr) => {

                match expr {
                    Expr::StringLiteral(s) => {

                        let label = self.intern_string(s);

                        self.emit(format!("\tadr x0, fmt_str         // load address of format string \"%s\\n\""));
                        self.emit(format!("\tadr x1, {}             // address of the string literal", label));
                        self.emit("\tbl printf                 // call printf(fmt_str, string)");
                    }
                    _ => {
                        let reg = self.gen_expr(expr);
                        //
                        self.emit("\tadr x0, fmt_int         // load address of \"%d\\n\" format");
                        self.emit(format!("\tmov x1, {}              // move value to x1 (printf 2nd arg)", reg));
                        self.emit("\tbl printf                 // call printf(fmt_int, value)");
                        self.free_tmp(reg);
                    }
                }
            }
            Stmt::If { condition, then_block, else_block } => {
                let cond_reg = self.gen_expr(condition);


                let else_lbl = self.fresh_label("else");
                let end_lbl = self.fresh_label("end_if");


                self.emit(format!("\tcmp {}, #0               // compare condition with 0", cond_reg));
                self.emit(format!("\tbeq {}                 // if equal (false) branch to else", else_lbl));

                for s in then_block {
                    self.gen_stmt(s);
                }
                self.emit(format!("\tb {}                    // jump to end_if after then", end_lbl));

                self.emit(format!("\t{}:                      // else label", else_lbl));
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.gen_stmt(s);
                    }
                }
                self.emit(format!("\t{}:                      // end_if label", end_lbl));
                self.free_tmp(cond_reg);
            }
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> &'static str {
        match expr {
            Expr::IntegerLiteral(n) => {
                let reg = self.alloc_tmp().expect("out of temporary registers");
                
                self.emit(format!("\tmov {}, #{}        // literal {}", reg, n, n));
                reg
            }
            Expr::BooleanLiteral(b) => {
                let reg = self.alloc_tmp().expect("out of temporary registers");
                let val = if *b { 1 } else { 0 };
                self.emit(format!("\tmov {}, #{}        // boolean literal {}", reg, val, val));
                reg
            }
            Expr::StringLiteral(s) => {
                let label = self.intern_string(s);
                let reg = self.alloc_tmp().expect("out of temporary registers");
                self.emit(format!("\tadr {}, {}         // address of string literal", reg, label));
                reg
            }
            Expr::Identifier(name) => {
                let reg = self.alloc_tmp().expect("out of temporary registers");
                if let Some(off) = self.lookup_var(name) {
                    self.emit(format!("\tldr {}, [sp, #{}]    // load variable '{}'", reg, off, name));
                } else {
                    self.emit(format!("\t// WARNING: variable '{}' not found; using 0", name));
                    self.emit(format!("\tmov {}, #0", reg));
                }
                reg
            }
            Expr::Assign { name, value } => {
                let val_reg = self.gen_expr(value);
                let off = self.allocate_var(name);
                self.emit(format!("\tstr {}, [sp, #{}]    // assign to '{}'", val_reg, off, name));
                val_reg
            }
            Expr::Binary { left, op, right } => {
                let r_left = self.gen_expr(left);
                let r_right = self.gen_expr(right);

                match op {
                    BinOp::Add => {
                        self.emit(format!("\tadd {}, {}, {}    // {} + {}", r_left, r_left, r_right, r_left, r_right));
                        self.free_tmp(r_right);
                        r_left
                    }
                    BinOp::Sub => {
                        self.emit(format!("\tsub {}, {}, {}    // {} - {}", r_left, r_left, r_right, r_left, r_right));
                        self.free_tmp(r_right);
                        r_left
                    }
                    BinOp::GreaterThan => {
                        // cmp left, right ; cset dst, gt
                        self.emit(format!("\tcmp {}, {}          // compare left and right", r_left, r_right));
                        self.emit(format!("\tcset {}, gt         // set {} = (left > right) ? 1 : 0", r_left, r_left));
                        self.free_tmp(r_right);
                        r_left
                    }
                    BinOp::LessThan => {
                        self.emit(format!("\tcmp {}, {}          // compare left and right", r_left, r_right));
                        self.emit(format!("\tcset {}, lt         // set {} = (left < right) ? 1 : 0", r_left, r_left));
                        self.free_tmp(r_right);
                        r_left
                    }
                }
            }
        }
    }
}

