#[derive(Debug)]
pub enum Token {
    
    // keywords
    Print(String),
    If(String),
    Else(String),
    Int(String),
    Minus(String),

    // literals
    IntegerLiteral(i32),
    StringLiteral(String),
    BooleanLiteral(bool),

    // identifiers
    Identifier(String),

    // operators 
    Plus(String),
    Assign(String),
    
    // punctuation
    SemiColon(String),
    LeftParen(String),
    RightParen(String),
    LeftBrace(String),
    RightBrace(String),

    // logical operators
    GreaterThan(String),
    LessThan(String),
}

impl Token {
    pub fn get_token(token_type: &str, value: Option<&str>) -> Token {
        match token_type {

            // keywords
            "Print" => Token::Print("print".to_string()),
            "If" => Token::If("if".to_string()),
            "Else" => Token::Else("else".to_string()),
            "Int" => Token::Int("int".to_string()),

            // literals
            "IntegerLiteral" => {
                Token::IntegerLiteral(value.unwrap().parse::<i32>().unwrap())
            }
            "StringLiteral" => {
                Token::StringLiteral(value.unwrap().to_string())
            }
            "BooleanLiteral" => {
                let val = match value.unwrap() {
                    "true" => true,
                    "false" => false,
                    _ => panic!("invalid boolean literal"),
                };
                Token::BooleanLiteral(val)
            }

            // identifiers
            "Identifier" => Token::Identifier(value.unwrap().to_string()),

            // operators
            "Plus" => Token::Plus("+".to_string()),
            "Minus" => Token::Minus("-".to_string()),
            "Assign" => Token::Assign("=".to_string()),

            // punctuation
            "SemiColon" => Token::SemiColon(";".to_string()),
            "LeftParen" => Token::LeftParen("(".to_string()),
            "RightParen" => Token::RightParen(")".to_string()),
            "LeftBrace" => Token::LeftBrace("{".to_string()),
            "RightBrace" => Token::RightBrace("}".to_string()),

            // logical operators
            "GreaterThan" => Token::GreaterThan(">".to_string()),
            "LessThan" => Token::LessThan("<".to_string()),

            _ => panic!("invalid token type {}", token_type),
        }
    }

    pub fn get_token_regex(token_type: &str) -> String {
        match token_type {

            // keywords
            "Print" => r"print",
            "If" => r"if",
            "Else" => r"else",
            "Int" => r"int\s+",

            // literals
            "IntegerLiteral" => r"\d+",
            "StringLiteral" => r#"\".*\""#,
            "BooleanLiteral" => r"\b(?:true|false)\b",

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
        }.to_string()
    }
}
