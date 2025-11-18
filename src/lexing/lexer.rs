use regex::Regex;
use crate::lexing::token::Token;

pub fn lex_program(program: &str) -> Vec<Token> {
    let current_input = program;

    let tokens = [
        "Print",
        "If",
        "Else",
        "Int",

        "BooleanLiteral",
        "IntegerLiteral",
        "StringLiteral",

        "Plus",
        "Minus",
        "Assign",
        "GreaterThan",
        "LessThan",

        "LeftParen",
        "RightParen",
        "LeftBrace",
        "RightBrace",
        "SemiColon",

        "Identifier", 
    ];

    let mut match_vec: Vec<(&str, usize, usize)> = Vec::new();

    for token in tokens.iter() {
        let token_regex = Token::get_token_regex(token);
        let re = Regex::new(&token_regex).unwrap();

        for m in re.find_iter(current_input) {
            match_vec.push((token, m.start(), m.end()));
        }
    }

    // Sort by:
    // 1. start position ascending
    // 2. length descending (longest match wins)
    match_vec.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| (b.2 - b.1).cmp(&(a.2 - a.1)))
    });

    let mut token_vec = Vec::new();
    let mut last_end = 0;

    for (token_type, start, end) in match_vec {
        // Skip overlapping tokens (common lexer conflict)
        if start < last_end {
            continue;
        }
        last_end = end;

        let lexeme = &current_input[start..end];
        token_vec.push(Token::get_token(token_type, Some(lexeme)));
    }

    token_vec
}
