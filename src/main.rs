mod lexing;

use lexing::lexer::lex_program;

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
    let tokens = lex_program(PROGRAM);

    for token in tokens.iter() {
        println!("{:?}", token);
    }
}
