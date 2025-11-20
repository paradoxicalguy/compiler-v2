mod lexing;
mod parser;
mod ast;
mod semantic;
mod codegen;

use lexing::lexer::lex_program;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::Codegen;

const PROGRAM: &str = "
    int x = 5;
    int y = 6;
    int z = x + y;

    if (z > 0) {
        print(\"hihi\");
    } else {
        print(\"haha\");
    }
";

fn main() {
    println!("--- LEXING ---");
    let tokens = lex_program(PROGRAM);
    for t in &tokens {
        println!("{:?}", t);
    }

    println!("\n--- PARSING ---");
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(stmts) => stmts,
        Err(e) => {
            println!("Parse error: {:?}", e);
            return;
        }
    };
    println!("AST = {:#?}", ast);

    println!("\n--- SEMANTIC ANALYSIS ---");
    let mut analyzer = SemanticAnalyzer::new();
    if let Err(errors) = analyzer.analyze(&ast) {
        println!("semantic errors:");
        for e in errors {
            println!("{:?}", e);
        }
        return;
    }
    println!("No semantic errors.");

    println!("\n--- CODE GENERATION (ARM64) ---");
    let asm = Codegen::new().generate(&ast);

    // 🔥 Print assembly in terminal
    println!("\n--- GENERATED ARM64 ASSEMBLY ---\n");
    println!("{}", asm);

    // also write to out.s
    let path = "out.s";
    std::fs::write(path, asm).expect("failed to write out.s");

    println!("\nAssembly also written to {}", path);
    println!("Compile with:");
    println!("    aarch64-linux-gnu-gcc out.s -o out");
}
