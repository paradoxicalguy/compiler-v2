
use std::collections::{HashMap, HashSet};
use crate::ast::{Expr, Stmt, BinOp};



#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    Int(i32),      // Integer constant: 5, 10, -3
    String(String), // String constant: "hello"
    Bool(bool),    // Boolean constant: true, false
}

pub struct Optimizer {
    // Maps variable names to their constant values (if known)
    // Example: { "x" -> ConstValue::Int(5), "y" -> ConstValue::Int(10) }
    constants: HashMap<String, ConstValue>,
    
    // Set of variables that are actually USED in the program
    // We track this to eliminate unused variables (dead code)
    used_variables: HashSet<String>,
}

impl Optimizer {
    // ========================================================================
    // CONSTRUCTOR
    // ========================================================================
    
    pub fn new() -> Self {
        Optimizer {
            constants: HashMap::new(),
            used_variables: HashSet::new(),
        }
    }
    
    // ========================================================================
    // MAIN OPTIMIZATION ENTRY POINT
    // ========================================================================
    
    // Optimize a program (list of statements)
    // Runs multiple passes until nothing changes anymore
    pub fn optimize(&mut self, statements: Vec<Stmt>) -> Vec<Stmt> {
        let mut current = statements;
        
        // We run optimization in a loop because:
        // - One optimization might enable another
        // - Example: constant folding enables constant propagation
        // We stop when nothing changes (reached a "fixed point")
        
        let mut iteration = 0;
        loop {
            iteration += 1;
            
            // Save the current state to detect if anything changed
            let before = format!("{:?}", current);
            
            // Pass 1: Collect which variables are actually used
            // This helps us identify dead code
            self.used_variables.clear();
            self.collect_used_variables(&current);
            
            // Pass 2: Constant folding and propagation
            // Walk through and simplify expressions
            current = self.optimize_statements(current);
            
            // Pass 3: Dead code elimination
            // Remove variables that are never used
            current = self.eliminate_dead_code(current);
            
            let after = format!("{:?}", current);
            
            // If nothing changed, we're done optimizing!
            if before == after {
                break;
            }
            
            // Safety limit: don't run forever
            // (In case we have a bug that causes infinite optimization)
            if iteration > 100 {
                eprintln!("Warning: Optimization didn't converge after 100 iterations");
                break;
            }
        }
        
        current
    }
    
    // ========================================================================
    // STATEMENT OPTIMIZATION
    // ========================================================================
    
    // Optimize a list of statements
    fn optimize_statements(&mut self, statements: Vec<Stmt>) -> Vec<Stmt> {
        let mut optimized = Vec::new();
        
        for stmt in statements {
            if let Some(opt_stmt) = self.optimize_statement(stmt) {
                optimized.push(opt_stmt);
            }
            // If optimize_statement returns None, the statement was eliminated
        }
        
        optimized
    }
    
    // Optimize a single statement
    // Returns None if the statement should be eliminated (dead code)
    fn optimize_statement(&mut self, stmt: Stmt) -> Option<Stmt> {
        match stmt {
            // Variable declaration: int x = expression;
            Stmt::VarDeclaration { name, value } => {
                // Step 1: Optimize the value expression
                let optimized_value = self.optimize_expression(value);
                
                // Step 2: If the value is a constant, remember it!
                // This enables constant propagation later
                if let Some(const_val) = self.try_evaluate_to_const(&optimized_value) {
                    self.constants.insert(name.clone(), const_val);
                } else {
                    // Value is not constant, so forget any previous constant value
                    self.constants.remove(&name);
                }
                
                // Return the optimized declaration
                Some(Stmt::VarDeclaration {
                    name,
                    value: optimized_value,
                })
            }
            
            // Print statement: print(expression);
            Stmt::Print(expr) => {
                // Optimize the expression being printed
                let optimized_expr = self.optimize_expression(expr);
                Some(Stmt::Print(optimized_expr))
            }
            
            // If statement: if (condition) { ... } else { ... }
            Stmt::If { condition, then_block, else_block } => {
                self.optimize_if_statement(condition, then_block, else_block)
            }
        }
    }
    
    // Optimize an if statement
    // This is complex because we might be able to eliminate entire branches!
    fn optimize_if_statement(
        &mut self,
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    ) -> Option<Stmt> {
        // Step 1: Optimize the condition
        let optimized_condition = self.optimize_expression(condition);
        
        // Step 2: Check if condition is a constant (true/false)
        // If it is, we can eliminate one of the branches!
        if let Some(const_val) = self.try_evaluate_to_const(&optimized_condition) {
            match const_val {
                ConstValue::Bool(true) => {
                    // Condition is always true!
                    // Only the then_block will ever execute
                    // So we can replace the entire if with just the then_block
                    let optimized_then = self.optimize_statements(then_block);
                    
                    // IMPORTANT: We can't just return the statements directly
                    // We need to keep them as a block or flatten them
                    // For simplicity, we'll keep the if statement but mark it as always true
                    // A more advanced optimizer would flatten this
                    return Some(Stmt::If {
                        condition: optimized_condition,
                        then_block: optimized_then,
                        else_block: None, // else never runs, so eliminate it
                    });
                }
                ConstValue::Bool(false) => {
                    // Condition is always false!
                    // Only the else_block will execute (if it exists)
                    if let Some(else_stmts) = else_block {
                        let optimized_else = self.optimize_statements(else_stmts);
                        // Similar to above - keep as if for now
                        return Some(Stmt::If {
                            condition: optimized_condition,
                            then_block: vec![],
                            else_block: Some(optimized_else),
                        });
                    } else {
                        // No else block and condition is false
                        // The entire if statement does nothing!
                        return None; // Eliminate it completely
                    }
                }
                _ => {
                    // Condition is a constant but not a boolean
                    // This is a semantic error, but we'll let it through
                    // (semantic analyzer should have caught this)
                }
            }
        }
        
        // Step 3: Condition is not constant, so optimize both branches
        let optimized_then = self.optimize_statements(then_block);
        let optimized_else = else_block.map(|stmts| self.optimize_statements(stmts));
        
        Some(Stmt::If {
            condition: optimized_condition,
            then_block: optimized_then,
            else_block: optimized_else,
        })
    }
    
    // ========================================================================
    // EXPRESSION OPTIMIZATION
    // ========================================================================
    
    // Optimize an expression
    // This is where the magic happens!
    fn optimize_expression(&self, expr: Expr) -> Expr {
        match expr {
            // Literals are already optimal - can't improve them
            Expr::IntegerLiteral(_) | Expr::StringLiteral(_) => expr,
            
            // Variable reference: might be able to replace with constant!
            Expr::Identifier(name) => {
                // CONSTANT PROPAGATION:
                // If we know this variable's value, replace it!
                if let Some(const_val) = self.constants.get(&name) {
                    match const_val {
                        ConstValue::Int(n) => Expr::IntegerLiteral(*n),
                        ConstValue::String(s) => Expr::StringLiteral(s.clone()),
                        ConstValue::Bool(_b) => {
                            // We don't have boolean literals in our AST yet
                            // So keep it as identifier for now
                            Expr::Identifier(name)
                        }
                    }
                } else {
                    // Value unknown, keep as variable reference
                    Expr::Identifier(name)
                }
            }
            
            // Binary operation: left op right
            Expr::Binary { left, op, right } => {
                self.optimize_binary_expression(*left, op, *right)
            }
            
            // Assignment: name = value
            Expr::Assign { name, value } => {
                // Optimize the value being assigned
                let optimized_value = self.optimize_expression(*value);
                Expr::Assign {
                    name,
                    value: Box::new(optimized_value),
                }
            }
        }
    }
    
    // Optimize a binary expression: left op right
    // This does CONSTANT FOLDING and ALGEBRAIC SIMPLIFICATION
    fn optimize_binary_expression(&self, left: Expr, op: BinOp, right: Expr) -> Expr {
        // Step 1: Recursively optimize both sides
        let opt_left = self.optimize_expression(left);
        let opt_right = self.optimize_expression(right);
        
        // Step 2: Try to evaluate as constants (CONSTANT FOLDING)
        let left_const = self.try_evaluate_to_const(&opt_left);
        let right_const = self.try_evaluate_to_const(&opt_right);
        
        // If both sides are constants, we can compute the result now!
        if let (Some(l), Some(r)) = (&left_const, &right_const) {
            if let Some(result) = self.fold_binary_operation(l, &op, r) {
                return result;
            }
        }
        
        // Step 3: Algebraic simplifications (even if not fully constant)
        // These are mathematical identities that always hold
        
        match op {
            BinOp::Add => {
                // x + 0 = x
                if let Some(ConstValue::Int(0)) = right_const {
                    return opt_left;
                }
                // 0 + x = x
                if let Some(ConstValue::Int(0)) = left_const {
                    return opt_right;
                }
            }
            BinOp::Sub => {
                // x - 0 = x
                if let Some(ConstValue::Int(0)) = right_const {
                    return opt_left;
                }
                // x - x = 0 (if both sides are the same identifier)
                if let (Expr::Identifier(l), Expr::Identifier(r)) = (&opt_left, &opt_right) {
                    if l == r {
                        return Expr::IntegerLiteral(0);
                    }
                }
            }
            _ => {}
        }
        
        // Step 4: Can't optimize further, return the expression
        Expr::Binary {
            left: Box::new(opt_left),
            op,
            right: Box::new(opt_right),
        }
    }
    
    // ========================================================================
    // CONSTANT FOLDING HELPERS
    // ========================================================================
    
    // Try to evaluate an expression to a compile-time constant
    // Returns Some(value) if it's constant, None if it depends on runtime values
    fn try_evaluate_to_const(&self, expr: &Expr) -> Option<ConstValue> {
        match expr {
            Expr::IntegerLiteral(n) => Some(ConstValue::Int(*n)),
            Expr::StringLiteral(s) => Some(ConstValue::String(s.clone())),
            
            Expr::Identifier(name) => {
                // Look up if this variable has a known constant value
                self.constants.get(name).cloned()
            }
            
            Expr::Binary { left, op, right } => {
                // Try to evaluate both sides
                let left_val = self.try_evaluate_to_const(left)?;
                let right_val = self.try_evaluate_to_const(right)?;
                
                // Fold the operation
                self.fold_binary_operation(&left_val, op, &right_val)
                    .and_then(|expr| self.try_evaluate_to_const(&expr))
            }
            
            Expr::Assign { .. } => {
                // Assignments are not constant expressions
                None
            }
        }
    }
    
    // Perform a binary operation on constant values at compile time
    // Returns an expression with the computed result
    fn fold_binary_operation(
        &self,
        left: &ConstValue,
        op: &BinOp,
        right: &ConstValue,
    ) -> Option<Expr> {
        match (left, op, right) {
            // Integer operations
            (ConstValue::Int(l), BinOp::Add, ConstValue::Int(r)) => {
                Some(Expr::IntegerLiteral(l + r))
            }
            (ConstValue::Int(l), BinOp::Sub, ConstValue::Int(r)) => {
                Some(Expr::IntegerLiteral(l - r))
            }
            (ConstValue::Int(l), BinOp::GreaterThan, ConstValue::Int(r)) => {
                // Result is boolean, but we don't have boolean literals yet
                // So we'll represent as 1 (true) or 0 (false)
                // TODO: Once you add boolean literals to AST, return those instead
                let result = if l > r { 1 } else { 0 };
                Some(Expr::IntegerLiteral(result))
            }
            (ConstValue::Int(l), BinOp::LessThan, ConstValue::Int(r)) => {
                let result = if l < r { 1 } else { 0 };
                Some(Expr::IntegerLiteral(result))
            }
            
            // String operations
            (ConstValue::String(l), BinOp::Add, ConstValue::String(r)) => {
                // String concatenation
                Some(Expr::StringLiteral(format!("{}{}", l, r)))
            }
            
            // Can't fold this combination
            _ => None,
        }
    }
    
    // ========================================================================
    // DEAD CODE ELIMINATION
    // ========================================================================
    
    // Collect all variables that are USED (referenced) in the program
    // This helps us identify dead code (variables that are declared but never used)
    fn collect_used_variables(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.collect_used_in_statement(stmt);
        }
    }
    
    fn collect_used_in_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDeclaration { name: _, value } => {
                // The variable being declared is NOT considered "used"
                // But the value expression might use other variables
                self.collect_used_in_expression(value);
            }
            Stmt::Print(expr) => {
                self.collect_used_in_expression(expr);
            }
            Stmt::If { condition, then_block, else_block } => {
                self.collect_used_in_expression(condition);
                for stmt in then_block {
                    self.collect_used_in_statement(stmt);
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.collect_used_in_statement(stmt);
                    }
                }
            }
        }
    }
    
    fn collect_used_in_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::IntegerLiteral(_) | Expr::StringLiteral(_) => {
                // Literals don't use variables
            }
            Expr::Identifier(name) => {
                // This variable is used!
                self.used_variables.insert(name.clone());
            }
            Expr::Binary { left, right, .. } => {
                self.collect_used_in_expression(left);
                self.collect_used_in_expression(right);
            }
            Expr::Assign { name, value } => {
                // Assignment TARGET is considered "used"
                // (because we're writing to it)
                self.used_variables.insert(name.clone());
                self.collect_used_in_expression(value);
            }
        }
    }
    
    // Remove statements that declare variables that are never used
    fn eliminate_dead_code(&self, statements: Vec<Stmt>) -> Vec<Stmt> {
        statements
            .into_iter()
            .filter_map(|stmt| match stmt {
                Stmt::VarDeclaration { ref name, .. } => {
                    // If this variable is never used, eliminate it!
                    if self.used_variables.contains(name) {
                        Some(stmt) // Keep it
                    } else {
                        None // Dead code - eliminate it!
                    }
                }
                // Keep all other statements
                _ => Some(stmt),
            })
            .collect()
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================
/*
use your_crate::{Parser, SemanticAnalyzer, Optimizer};

fn main() {
    // Parse the program
    let tokens = vec![...];
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    
    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast).unwrap();
    
    // Optimize!
    let mut optimizer = Optimizer::new();
    let optimized_ast = optimizer.optimize(ast);
    
    println!("Original AST: {:#?}", ast);
    println!("Optimized AST: {:#?}", optimized_ast);
}
*/

// ============================================================================
// DETAILED WALKTHROUGH - HOW OPTIMIZATION WORKS
// ============================================================================
/*

Let's trace through optimizing this program:

int x = 5 + 3;
int y = x + 2;
int z = 10;
print(y);

=== ITERATION 1 ===

--- Pass 1: Collect Used Variables ---
Walk through and find: y is used in print(y)
used_variables = {"y"}

--- Pass 2: Optimize Statements ---

Statement 1: int x = 5 + 3;
  - optimize_expression(5 + 3)
    - optimize_binary_expression(5, Add, 3)
    - Both sides are constants: ConstValue::Int(5), ConstValue::Int(3)
    - fold_binary_operation: 5 + 3 = 8
    - Returns: IntegerLiteral(8)
  - Value is constant: ConstValue::Int(8)
  - Store in constants: {"x" -> ConstValue::Int(8)}
  - Result: int x = 8;

Statement 2: int y = x + 2;
  - optimize_expression(x + 2)
    - optimize_binary_expression(Identifier("x"), Add, 2)
    - Optimize left: Identifier("x")
      - Look up in constants: found ConstValue::Int(8)
      - CONSTANT PROPAGATION: Replace with IntegerLiteral(8)
    - Optimize right: IntegerLiteral(2) (already optimal)
    - Both sides are now constants: 8 and 2
    - fold_binary_operation: 8 + 2 = 10
    - Returns: IntegerLiteral(10)
  - Value is constant: ConstValue::Int(10)
  - Store in constants: {"x" -> 8, "y" -> 10}
  - Result: int y = 10;

Statement 3: int z = 10;
  - optimize_expression(10) -> IntegerLiteral(10)
  - Value is constant: ConstValue::Int(10)
  - Store in constants: {"x" -> 8, "y" -> 10, "z" -> 10}
  - Result: int z = 10;

Statement 4: print(y);
  - optimize_expression(Identifier("y"))
  - Look up in constants: found ConstValue::Int(10)
  - CONSTANT PROPAGATION: Replace with IntegerLiteral(10)
  - Result: print(10);

After Pass 2:
int x = 8;
int y = 10;
int z = 10;
print(10);

--- Pass 3: Dead Code Elimination ---
used_variables = {"y"}
- x is declared but not in used_variables -> ELIMINATE
- y is in used_variables -> KEEP
- z is declared but not in used_variables -> ELIMINATE

After Pass 3:
int y = 10;
print(10);

=== ITERATION 2 ===

--- Pass 1: Collect Used Variables ---
used_variables = {} (empty - no variables used anymore!)

--- Pass 2: Optimize Statements ---
Nothing changes

--- Pass 3: Dead Code Elimination ---
- y is declared but not used -> ELIMINATE

After Pass 3:
print(10);

=== ITERATION 3 ===
Nothing changes -> DONE!

=== FINAL OPTIMIZED CODE ===
print(10);

We went from:
  int x = 5 + 3;
  int y = x + 2;
  int z = 10;
  print(y);

To:
  print(10);

Same result, but MUCH simpler and faster! 🚀

*/