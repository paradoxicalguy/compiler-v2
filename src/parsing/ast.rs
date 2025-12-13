#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntegerLiteral(i32),
    StringLiteral(String),
    BooleanLiteral(bool), 
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Expr),
    VarDeclaration {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
     Block(Vec<Stmt>),
    ExprStmt(Expr),
}


// first we create an enum of Expr, which has all type of Expr we can face in the code and 
// a binary Expr which has operators in it, then a statement enum which has print and it takes Expr as 
// its input.
// variable declaration; which declares any variables based on their names and their values,it stores names as strings
// and value as any of the Expr we defined above.
// if block is where the condition is defined aka the Expr, then it moves to then_block if the defined 
// Expr is true or to else_block of the Expr is untrue.
// 
// 