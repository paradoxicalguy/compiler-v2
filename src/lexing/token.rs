#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // keywords
    Print,
    If,
    Else,
    Int,
    Maybe,
    Paywall,

    // identifiers & literals
    Identifier(String),
    IntegerLiteral(i64),
    StringLiteral(String),

    // operators
    Plus,
    Minus,
    Assign,
    GreaterThan,
    LessThan,

    // punctuation
    SemiColon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

impl Token {
    pub fn get_token(token_type: &str, value: Option<&str>) -> Token {
        match token_type {
            // keywords
            "Print" => Token::Print,
            "If" => Token::If,
            "Else" => Token::Else,
            "Int" => Token::Int,
            "Maybe" => Token::Maybe,
            "Paywall" => Token::Paywall,

            // literals
            "IntegerLiteral" => {
                Token::IntegerLiteral(
                    value
                        .expect("IntegerLiteral requires a value")
                        .parse::<i64>()
                        .expect("Invalid integer literal"),
                )
            }
            "StringLiteral" => {
                Token::StringLiteral(
                    value
                        .expect("StringLiteral requires a value")
                        .to_string(),
                )
            }

            // identifiers
            "Identifier" => {
                Token::Identifier(
                    value
                        .expect("Identifier requires a value")
                        .to_string(),
                )
            }

            // operators
            "Plus" => Token::Plus,
            "Minus" => Token::Minus,
            "Assign" => Token::Assign,

            // punctuation
            "SemiColon" => Token::SemiColon,
            "LeftParen" => Token::LeftParen,
            "RightParen" => Token::RightParen,
            "LeftBrace" => Token::LeftBrace,
            "RightBrace" => Token::RightBrace,

            // logical operators
            "GreaterThan" => Token::GreaterThan,
            "LessThan" => Token::LessThan,

            _ => panic!("invalid token type {}", token_type),
        }
    }

    pub fn get_token_regex(token_type: &str) -> String {
        match token_type {
            // keywords
            "Print" => r"\bprint\b",
            "If" => r"\bif\b",
            "Else" => r"\belse\b",
            "Int" => r"\bint\b",
            "Maybe" => r"\bmaybe\b",
            "Paywall" => r"\bpaywall\b",

            // literals
            "IntegerLiteral" => r"\d+",
            "StringLiteral" => r#""[^"]*""#,

            // identifiers
            "Identifier" => r"[a-zA-Z_][a-zA-Z0-9_]*",

            // operators
            "Plus" => r"\+",
            "Minus" => r"-",
            "Assign" => r"=",

            // punctuation
            "SemiColon" => r";",
            "LeftParen" => r"\(",
            "RightParen" => r"\)",
            "LeftBrace" => r"\{",
            "RightBrace" => r"\}",

            // logical operators
            "GreaterThan" => r">",
            "LessThan" => r"<",

            _ => panic!("invalid token type: {}", token_type),
        }
        .to_string()
    }
}
