use regex::Regex;
use crate::lexing::token::Token;

pub fn lex_program(program: &str) -> Vec<Token> {
    let tokens = [
        // keywords
        "Print",
        "If",
        "Else",
        "Int",

        // literals
        "IntegerLiteral",
        "StringLiteral",

        // operators
        "Plus",
        "Minus",
        "Assign",
        "GreaterThan",
        "LessThan",

        // punctuation
        "LeftParen",
        "RightParen",
        "LeftBrace",
        "RightBrace",
        "SemiColon",

        // identifiers (keep LAST)
        "Identifier",
    ];

    let mut matches: Vec<(&str, usize, usize)> = Vec::new();

    for token_type in tokens {
        let regex = Regex::new(&Token::get_token_regex(token_type))
            .expect("invalid regex");

        for m in regex.find_iter(program) {
            matches.push((token_type, m.start(), m.end()));
        }
    }

    // sort by position, then longest match first
    matches.sort_by(|a, b| {
        a.1.cmp(&b.1)
            .then_with(|| (b.2 - b.1).cmp(&(a.2 - a.1)))
    });

    let mut result = Vec::new();
    let mut last_end = 0;

    for (token_type, start, end) in matches {
        if start < last_end {
            continue;
        }
        last_end = end;

        let lexeme = &program[start..end];

        let token = match token_type {
            // keywords 
            "Print" => Token::Print,
            "If" => Token::If,
            "Else" => Token::Else,
            "Int" => Token::Int,

            //  literals
            "IntegerLiteral" => {
                let value = lexeme.parse::<i64>().unwrap();
                Token::IntegerLiteral(value)
            }

            "StringLiteral" => {
                // remove surrounding quotes
                let inner = &lexeme[1..lexeme.len() - 1];
                Token::StringLiteral(inner.to_string())
            }   

            // identifiers
            "Identifier" => Token::Identifier(lexeme.to_string()),

            // operators 
            "Plus" => Token::Plus,
            "Minus" => Token::Minus,
            "Assign" => Token::Assign,
            "GreaterThan" => Token::GreaterThan,
            "LessThan" => Token::LessThan,

            // punctuation 
            "SemiColon" => Token::SemiColon,
            "LeftParen" => Token::LeftParen,
            "RightParen" => Token::RightParen,
            "LeftBrace" => Token::LeftBrace,
            "RightBrace" => Token::RightBrace,

            _ => unreachable!("unknown token type"),
        };

        result.push(token);
    }

    result
}
