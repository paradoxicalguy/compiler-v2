mod lexing;
mod parser;
mod ast;
mod semantic;
mod codegen;
use std::time::Instant;
use std::process::Command;

use lexing::lexer::lex_program;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::Codegen;

/// Generate a large program by repeating a scoped block.
/// Each repetition is wrapped in `{}` to avoid redeclaration errors.
fn make_program(repetitions: usize) -> String {
    let block = r#"
    {
       int x = "69";
       int y = "420";
       int z = "x + y";
    }
    "#;

    block.repeat(repetitions)
}

fn main() {
    // ================= CONFIG =================
    let repetitions = 1; // try: 1, 10, 50, 100, 500
    let program = make_program(repetitions);

    println!("benchmarking with {} repeated blocks", repetitions);

    let total_start = Instant::now();

    // ================= LEXING =================
    let lex_start = Instant::now();
    let tokens = lex_program(&program);
    let lex_time = lex_start.elapsed();

    // ================= PARSING =================
    let parse_start = Instant::now();
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            println!("parse error: {:?}", e);
            return;
        }
    };
    let parse_time = parse_start.elapsed();

    // ================= SEMANTIC =================
    let semantic_start = Instant::now();
    let mut analyzer = SemanticAnalyzer::new();
    let semantic_result = analyzer.analyze(&ast);
    let semantic_time = semantic_start.elapsed();

    if let Err(errors) = &semantic_result {
        println!("semantic errors ({}):", errors.len());
        for e in errors {
            println!("  {:?}", e);
        }
        // IMPORTANT: do NOT return — keep benchmarking
    }

    // ================= CODEGEN =================
    let codegen_start = Instant::now();
    let asm = Codegen::new().generate(&ast);
    let codegen_time = codegen_start.elapsed();

    std::fs::write("out.s", &asm).expect("failed to write out.s");

    //  println!("\n========== GENERATED AARCH64 ASSEMBLY ==========\n");

    // // Prevent terminal nuking on huge outputs
    // let max_lines = 300;
    // for (i, line) in asm.lines().enumerate() {
    //     if i >= max_lines {
    //         println!("... (assembly truncated, {}+ lines total)", asm.lines().count());
    //         break;
    //     }
    //     println!("{}", line);
    // }

    // println!("\n========== END ASSEMBLY ==========\n");

    // std::fs::write("out.s", &asm).expect("failed to write out.s");

    // ================= ASSEMBLE =================
    let assemble_start = Instant::now();

    let assemble_status = Command::new("aarch64-linux-gnu-gcc")
        .args(["-static","out.s", "-o", "out"])
        .status();

    let assemble_time = assemble_start.elapsed();

    if assemble_status.is_err() || !assemble_status.unwrap().success() {
        println!("assembly failed");
        println!("\n--- TIMINGS ---");
        println!("Lexing:        {:?}", lex_time);
        println!("Parsing:       {:?}", parse_time);
        println!("Semantic:      {:?}", semantic_time);
        println!("Codegen:       {:?}", codegen_time);
        println!("Assemble:      FAILED");
        println!("Total:         {:?}", total_start.elapsed());
        return;
    }

    // ================= RUNTIME =================
    let run_start = Instant::now();

    let run_status = Command::new("qemu-aarch64")
    .args(["./out"])
    .status();

    let run_time = run_start.elapsed();

    if run_status.is_err() || !run_status.unwrap().success() {
        println!("runtime execution failed");
    }

    // ================= TIMINGS =================
    println!("\n--- TIMINGS ---");
    println!("Lexing:        {:?}", lex_time);
    println!("Parsing:       {:?}", parse_time);
    println!("Semantic:      {:?}", semantic_time);
    println!("Codegen:       {:?}", codegen_time);
    println!("Assemble:      {:?}", assemble_time);
    println!("Runtime:       {:?}", run_time);
    println!("Total:         {:?}", total_start.elapsed());

    println!("\nexecutable: out.exe");
}
