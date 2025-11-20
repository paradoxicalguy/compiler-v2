// file: src/codegen.rs
//
// Simple ARM64 code generator for your toy language.
// - Input: Vec<Stmt> (from your parser/AST)
// - Output: A string containing AArch64 assembly (text .s file)
// - Features supported: int vars, string literals, boolean, binary ops (Add, Sub, GreaterThan, LessThan),
//   assignments, print(expr), if/else.
//
// This generator is intentionally simple and very explicit so you can read the generated assembly
// and map it to the AST easily. It uses a small pool of temporary registers (x9..x15), and stores
// local variables on the stack in 8-byte slots (64-bit).
//
// Important: This emits `adr x0, label` to load addresses of static strings; most toolchains accept that.
// If your assembler complains, we can change to `adrp/add` or `ldr` literal pools.
//
// Usage (example):
// let mut cg = Codegen::new();
// let asm = cg.generate(&stmts);
// std::fs::write("out.s", asm).unwrap();
//
// NOTE: This code expects your AST types in crate::ast: Expr, Stmt, BinOp
use std::collections::HashMap;
use crate::ast::{Expr, Stmt, BinOp};

/// Codegen struct holds state during generation:
/// - data section strings (label -> value)
/// - variable symbol table (name -> stack offset)
/// - next stack offset to allocate
/// - temp register pool
/// - instruction buffer (Vec<String>) where we push text lines
/// - label counter for unique labels
pub struct Codegen {
    data_strings: Vec<(String, String)>,            // (label, text) for .data
    var_offsets: HashMap<String, i32>,              // variable -> offset from current sp
    next_var_offset: i32,                           // next offset (in bytes) to allocate (0, 8, 16, ...)
    instrs: Vec<String>,                            // emitted assembly lines (text)
    label_counter: usize,                           // for unique labels
    temp_regs: Vec<&'static str>,                   // available temporaries: x9..x15
}

/// Public API
impl Codegen {
    pub fn new() -> Self {
        Codegen {
            data_strings: Vec::new(),
            var_offsets: HashMap::new(),
            next_var_offset: 0,
            instrs: Vec::new(),
            label_counter: 0,
            // a small pool of caller-scratch registers we use for expr evaluation
            temp_regs: vec!["x9","x10","x11","x12","x13","x14","x15"],
        }
    }

    /// Main entry: generate assembly for program (list of statements).
    /// Returns full assembly as a single String (contains .data and .text sections).
    pub fn generate(mut self, stmts: &[Stmt]) -> String {
        // Walk the AST and emit code into self.instrs and self.data_strings
        self.emit_prologue();

        for s in stmts {
            self.gen_stmt(s);
        }

        self.emit_epilogue();

        // Build .data section with strings
        let mut data_section = Vec::new();
        // Add printf format strings (for ints and strings)
        // We'll use "%d\n" for integers, "%s\n" for strings.
        data_section.push(r#"fmt_int: .asciz "%d\n""#.to_string());
        data_section.push(r#"fmt_str: .asciz "%s\n""#.to_string());
        // add user strings
        for (label, text) in &self.data_strings {
            // text is already escaped
            data_section.push(format!("{}: .asciz {}", label, text));
        }

        // assemble final file
        let mut out = String::new();
        out.push_str("\t.data\n");
        for line in data_section {
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("\n\t.text\n");
        // global main
        out.push_str("\t.global main\n");
        // append instructions
        for instr in &self.instrs {
            out.push_str(instr);
            out.push('\n');
        }
        out
    }

    // -------------------------
    // Helpers and low-level emitters
    // -------------------------

    /// Emit a line of assembly (keeps it in buffer)
    fn emit(&mut self, line: impl Into<String>) {
        self.instrs.push(line.into());
    }

    /// Get a fresh unique label
    fn fresh_label(&mut self, base: &str) -> String {
        let lbl = format!("{}_{}", base, self.label_counter);
        self.label_counter += 1;
        lbl
    }

    /// Intern a string literal into the data section and return its label.
    /// `s` should be the literal content (including quotes will be added/kept).
    /// We will store it as an .asciz entry. We must keep the quotes as the assembler expects a string token.
    fn intern_string(&mut self, s: &str) -> String {
        // We expect s to be the lexeme like "\"hi world\"" including quotes.
        // To be safe, keep as-is. Make a label.
        let label = format!(".LC{}", self.data_strings.len());
        self.data_strings.push((label.clone(), s.to_string()));
        label
    }

    /// Allocate a new local variable on the stack.
    /// Returns the offset where the var resides (offset from sp).
    /// We assign offsets in 8-byte slots: 0, 8, 16, ...
    fn allocate_var(&mut self, name: &str) -> i32 {
        if let Some(off) = self.var_offsets.get(name) {
            return *off;
        }
        let off = self.next_var_offset;
        self.var_offsets.insert(name.to_string(), off);
        self.next_var_offset += 8;
        off
    }

    /// Lookup variable offset. Assumes variable exists.
    fn lookup_var(&self, name: &str) -> Option<i32> {
        self.var_offsets.get(name).copied()
    }

    /// Allocate a temporary register from the pool.
    /// Returns Some(reg) or None if pool empty.
    fn alloc_tmp(&mut self) -> Option<&'static str> {
        self.temp_regs.pop()
    }

    /// Free a temporary register back to the pool.
    fn free_tmp(&mut self, reg: &'static str) {
        self.temp_regs.push(reg);
    }

    // -------------------------
    // Emit program prologue/epilogue
    // -------------------------
    fn emit_prologue(&mut self) {
        // Standard prologue for main:
        // stp x29, x30, [sp, #-16]!  ; push frame pointer and return address
        // mov x29, sp                ; set frame pointer
        // sub sp, sp, #<n>          ; reserve stack for locals (we don't know n yet)
        //
        // We will reserve a slightly larger area after we see all variable allocations.
        // For simplicity, reserve 512 bytes up front (plenty for toy programs).
        //
        self.emit("\tmain:"); // label
        self.emit("\tstp x29, x30, [sp, #-16]!    // save fp and lr, push 16 bytes");
        self.emit("\tmov x29, sp                  // set frame pointer");
        // Reserve 512 bytes for locals (must keep 16-byte alignment)
        self.emit("\tsub sp, sp, #512            // reserve 512 bytes for locals (simple stack frame)");
        // We'll place variables starting at offset 0 relative to sp (sp + 0, sp + 8, ...)
        // The actual offsets used are small; 512 gives us headroom without complex resizing.
    }

    fn emit_epilogue(&mut self) {
        // Restore stack and return 0 from main
        // add sp, sp, #512
        // ldp x29, x30, [sp], #16
        // mov x0, #0
        // ret
        self.emit("\tadd sp, sp, #512            // deallocate frame");
        self.emit("\tldp x29, x30, [sp], #16     // restore fp and lr");
        self.emit("\tmov x0, #0                  // return 0");
        self.emit("\tret");
    }

    // -------------------------
    // Statement generation
    // -------------------------
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { name, value } => {
                // Evaluate initializer expression into a temp register, then store to var slot.
                // Allocate variable slot (8-byte aligned)
                let offset = self.allocate_var(name);
                // evaluate value
                let reg = self.gen_expr(value);
                // store reg into [sp, #offset]
                self.emit(format!("\tstr {}, [sp, #{}]    // store '{}' into local slot", reg, offset, name));
                self.free_tmp(reg);
            }
            Stmt::Print(expr) => {
                // Print supports strings and integers/booleans.
                match expr {
                    Expr::StringLiteral(s) => {
                        // string literal lexeme includes quotes, so use directly
                        let label = self.intern_string(s);
                        // x0 = format pointer (fmt_str), x1 = data pointer
                        self.emit(format!("\tadr x0, fmt_str         // load address of format string \"%s\\n\""));
                        self.emit(format!("\tadr x1, {}             // address of the string literal", label));
                        self.emit("\tbl printf                 // call printf(fmt_str, string)");
                    }
                    _ => {
                        // For non-string: evaluate expression into a register (int/bool)
                        let reg = self.gen_expr(expr);
                        // move reg into x1, set x0 to fmt_int
                        self.emit("\tadr x0, fmt_int         // load address of \"%d\\n\" format");
                        self.emit(format!("\tmov x1, {}              // move value to x1 (printf 2nd arg)", reg));
                        self.emit("\tbl printf                 // call printf(fmt_int, value)");
                        self.free_tmp(reg);
                    }
                }
            }
            Stmt::If { condition, then_block, else_block } => {
                // Evaluate condition -> get a register with 0/1 (false/true) or nonzero value.
                // We'll implement condition by evaluating the expression and comparing to zero.
                let cond_reg = self.gen_expr(condition);

                // create labels
                let else_lbl = self.fresh_label("else");
                let end_lbl = self.fresh_label("end_if");

                // if cond_reg == 0 -> jump to else
                self.emit(format!("\tcmp {}, #0               // compare condition with 0", cond_reg));
                self.emit(format!("\tbeq {}                 // if equal (false) branch to else", else_lbl));
                // THEN block
                for s in then_block {
                    self.gen_stmt(s);
                }
                self.emit(format!("\tb {}                    // jump to end_if after then", end_lbl));
                // ELSE label
                self.emit(format!("\t{}:                      // else label", else_lbl));
                if let Some(else_stmts) = else_block {
                    for s in else_stmts {
                        self.gen_stmt(s);
                    }
                }
                // End label
                self.emit(format!("\t{}:                      // end_if label", end_lbl));

                self.free_tmp(cond_reg);
            }
        }
    }

    // -------------------------
    // Expression generation
    // Returns: name of register (like "x9") that holds the result.
    // The caller is responsible for freeing that register by calling free_tmp(reg)
    // -------------------------
    fn gen_expr(&mut self, expr: &Expr) -> &'static str {
        match expr {
            Expr::IntegerLiteral(n) => {
                // allocate temp and mov immediate
                let reg = self.alloc_tmp().expect("out of temporary registers");
                // mov reg, #n
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
                // For string literal used in expressions, we return a register containing its address.
                let label = self.intern_string(s);
                let reg = self.alloc_tmp().expect("out of temporary registers");
                self.emit(format!("\tadr {}, {}         // address of string literal", reg, label));
                reg
            }
            Expr::Identifier(name) => {
                // load the variable from stack into a temp register
                let reg = self.alloc_tmp().expect("out of temporary registers");
                if let Some(off) = self.lookup_var(name) {
                    self.emit(format!("\tldr {}, [sp, #{}]    // load variable '{}'", reg, off, name));
                } else {
                    // variable not found: generate 0 and emit comment
                    self.emit(format!("\t// WARNING: variable '{}' not found; using 0", name));
                    self.emit(format!("\tmov {}, #0", reg));
                }
                reg
            }
            Expr::Assign { name, value } => {
                // Evaluate rhs, store into variable slot (like `name = value`) and return the value in a reg.
                let val_reg = self.gen_expr(value);
                // ensure var exists (allocate if needed) and store
                let off = self.allocate_var(name);
                self.emit(format!("\tstr {}, [sp, #{}]    // assign to '{}'", val_reg, off, name));
                // Keep the value in the register and return it (caller will free)
                val_reg
            }
            Expr::Binary { left, op, right } => {
                // Generate left and right into registers, emit operation, free right/left as appropriate.
                let r_left = self.gen_expr(left);
                let r_right = self.gen_expr(right);

                // we will place result into r_left (reuse left reg) and free r_right
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

// -------------------------
// Example helper: write assembly to file
// -------------------------
//
// Example usage (for your main.rs after parsing):
//
// use crate::codegen::Codegen;
// let cg = Codegen::new();
// let asm = cg.generate(&statements);
// std::fs::write("out.s", asm).unwrap();
//
// Then assemble & link on an aarch64 toolchain:
//   aarch64-linux-gnu-gcc out.s -o out   (cross-compile)
// Or run on native arm64 machine with `gcc out.s -o out`
//
// -------------------------

