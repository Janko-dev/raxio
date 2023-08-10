use std::{collections::HashMap, fs, error::Error};

use crate::{parser::{Expr, Stmt}, error::Warning};


pub struct Env {
    // History of all expressions after applying transformations.
    pub history: Vec<Expr>,

    // History of all expressions after applying transformations.
    pub derivation_history: Vec<(Expr, Expr, usize)>,
    
    // True if in pattern matching state and false if in global state
    pub is_matching: bool,

    // Rules hashmap from (string -> (lhs-expr, rhs-expr))
    pub rules: HashMap<String, (Expr, Expr)>,

    // Warnings that need to be printed to the user
    pub warnings: Vec<Warning>
}

impl Env {
    pub fn new() -> Self {
        Self { 
            history: vec![],
            derivation_history: vec![],
            is_matching: false, 
            rules: HashMap::new(),
            warnings: vec![]
        }
    }

    pub fn print_prefix(&self) {
        if self.is_matching {
            print!("    ~> ");
        } else {
            print!("> ");
        }
    }

    fn get_expr(&self) -> Option<&Expr> {
        if self.history.len() == 0 {
            None
        } else {
            self.history.get(self.history.len()-1)
        }
    }

    pub fn print_current_expr(&self, prefix: &str) {
        if let Some(expr) = self.get_expr() {
            println!("{}{}", prefix, expr.to_string());
            // For readability, also print as functor prefix notation
            if find_binary_ops(expr) {
                println!("{:indent$}As functor: {}", "", expr, indent=prefix.len());
            }
        }
    }

    pub fn pop_expr(&mut self) {
        if self.history.len() > 1 {
            self.history.pop();
            self.derivation_history.pop();
            self.print_current_expr("    ");
        }
    } 

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {

        // interpret each parsed statement.
        for stmt in stmts {
            // match on a statement and global/matching state.
            match (stmt, self.is_matching) {
                // These cases have no effect, and thus produce warnings
                (Stmt::ExprStmt(_), true) => self.warnings.push(Warning::ExprHasNoEffect),
                (Stmt::ApplyStmt { .. }, false) => self.warnings.push(Warning::ApplyRuleNoEffect),
                (Stmt::RuleStmt { .. }, false) => self.warnings.push(Warning::InLineRuleNoEffect),
                (Stmt::EndStmt(_), false) => self.warnings.push(Warning::EndStmtHasNoEffect),
                // If an expression is found, and we are not pattern matching
                // i.e., currently still in the global state, then start pattern matching.
                (Stmt::ExprStmt(expr), false) => {
                    self.is_matching = true;
                    self.history.push(expr);
                    self.print_current_expr("Start matching on: ");
                },
                // If an apply statement is found while in pattern matching state.
                (Stmt::ApplyStmt { iden, depth }, true) => {
                    // If variable identifier is a rule, then pattern match on the rule.
                    if self.rules.contains_key(&iden) {
                        
                        let (left, right) = self.rules.get(&iden).unwrap();
                        self.history.push(ast_traverse_match(
                            self.get_expr().unwrap().clone(), 
                            &left, 
                            &right,
                            depth,
                        )?);
                        self.derivation_history.push((left.to_owned(), right.to_owned(), depth));
                        self.print_current_expr("    ");
                    } else {
                        self.warnings.push(Warning::RuleDoesNotExist(iden));
                    }
                },
                // Define statements can be constructed in either global or matching state.
                (Stmt::DefineStmt { iden, left, right }, _) => {
                    self.rules.insert(iden, (left, right));
                },
                // In-line rule statements are directly mathed upon.
                (Stmt::RuleStmt { left, right, depth}, true) => {
                    self.history.push(ast_traverse_match(
                        self.get_expr().unwrap().clone(), 
                        &left, 
                        &right,
                        depth,
                    )?);
                    self.derivation_history.push((left, right, depth));
                    self.print_current_expr("    ");
                },
                (Stmt::EndStmt(path), true) => { 
                    self.print_current_expr("Result: ");
                    match path {
                        Some(file_path) => { self.write_to_file(file_path)?; },
                        None => {}
                    }
                    self.history.clear();
                    self.derivation_history.clear();
                    self.is_matching = false;
                },
            }
        }
        Ok(())
    }

    fn write_to_file(&mut self, file_path: String) -> Result<(), Box<dyn Error>> {
        let mut data = format!("Start pattern matching on {}\n", self.history.get(0).unwrap().to_string());
        data.push_str(
            &self.history
            .iter()
            .skip(1)
            .enumerate()
            .zip(self.derivation_history.iter())
            .map(|((i, expr), (lhs, rhs, depth))| {
                format!("\n{}. Applying rule: {} => {} at depth {}, results in:\n    {}\n", 
                    i+1, 
                    lhs.to_string(), 
                    rhs.to_string(),
                    depth,
                    expr.to_string() 
                )
            })
            .collect::<String>()
        );
        data.push_str(&format!("\nResult: {}", self.get_expr().unwrap().to_string()));

        fs::write(file_path, data)?;
        Ok(())
    }

}

// Traverse the Abstract Syntax Tree of the current expression, 
// and match sub-expression if and only if certain depth is reached.  
fn ast_traverse_match(current_expr: Expr, left: &Expr, right: &Expr, depth: usize) -> Result<Expr, Box<dyn Error>>{

    if depth == 0 {
        // Update current_expr by matching on left and producing corresponding right expression. 
        let current_expr = match_patterns(current_expr, left, right)?;
        Ok(current_expr)
    } else {
        match current_expr {
            cur @ Expr::Variable { .. } => Ok(cur),
            Expr::Functor { iden, args } => {
                let mut new_args = vec![];
                for arg in args {
                    let expr = ast_traverse_match(arg, left, right, depth - 1)?;
                    new_args.push(expr);
                }
                Ok(Expr::Functor { 
                    iden: iden,
                    args: new_args 
                })
            }
        }
    }
}

fn match_patterns(current_expr: Expr, left: &Expr, right: &Expr) -> Result<Expr, Box<dyn Error>>{

    match (current_expr, left) {
        (Expr::Variable { iden: current , ..}, 
         Expr::Variable { iden: lhs, .. }) => {
            if current.as_str() == lhs.as_str() {
                Ok(right.clone())
            } else {
                Ok(Expr::Variable { iden: current })
            }
        },
        (Expr::Functor { iden: current_iden, args: current_args },
         Expr::Functor { iden: lhs_iden, args: lhs_args }) => {
            // If both functors have the same arity and the same identifier
            // then they are considered to produce the form of the right expr. 
            if current_iden.as_str() == lhs_iden.as_str() &&
               current_args.len() == lhs_args.len()  
            {   
                let mut args_table = HashMap::<Expr, Expr>::new();
                // create mapping of (lhs args) -> (current_expr args)
                // return whether there is a match
                let is_match = fill_pattern_mapping(&current_args, lhs_args, &mut args_table);
                
                if is_match {
                    let res = construct_rhs(right, &args_table)?;
                    Ok(res)
                } else {
                    Ok(Expr::Functor { 
                        iden: current_iden, 
                        args: current_args 
                    })
                }
            } else {
                Ok(Expr::Functor { 
                    iden: current_iden, 
                    args: current_args 
                })
            }
        },
        // Cannot match variable against functor as the functor is a superset of the variable
        // i.e., contains more information. For instance, if current_expr conveys the symbol x 
        // and we try to match the rule f(x) => g(x), then we fail to match because f(x) != x. 
        (cur @ Expr::Variable { .. }, Expr::Functor { .. }) => Ok(cur),

        // In this case, we match current_expr (as a functor) against a variable.
        // This is possible as the functor may contain sub-expressions that match the left expr.
        (Expr::Functor { iden: current_iden, args: current_args }, 
        Expr::Variable { iden: lhs_iden, .. }) => {
            let mut new_args = vec![];
            for arg in current_args {
                if let Expr::Variable { iden } = arg {
                    if iden.as_str() == lhs_iden.as_str() {
                        new_args.push(right.clone());
                    } else {
                        new_args.push(Expr::Variable { iden });
                    }
                } else {
                    new_args.push(arg);
                }
            }
            Ok(Expr::Functor { 
                iden: current_iden, 
                args: new_args 
            })
        }
    }
}

// To fill the table of arguments, we recursively evaluate each sub-expression.
// This function also returns a bool indicating whether it is possible to construct the right hand side.
fn fill_pattern_mapping(cur_args: &Vec<Expr>, lhs_args: &Vec<Expr>, args_table: &mut HashMap<Expr, Expr>) -> bool {
    
    for (lhs_arg, cur_arg) in lhs_args.iter().zip(cur_args.iter())
    {
        match (lhs_arg, cur_arg) {
            (Expr::Variable { .. }, Expr::Variable { .. } | Expr::Functor { .. }) => {
                args_table.insert(lhs_arg.clone(), cur_arg.clone());
            },
            // current_expr: f(x)
            // f(g(x)) => ..
            (Expr::Functor { .. }, Expr::Variable { .. }) => {
                return false;
            },
            (Expr::Functor { iden: lhs_iden, args: _lhs_args }, 
             Expr::Functor { iden: cur_iden, args: _cur_args  }) => {
                if cur_iden.as_str() == lhs_iden.as_str() &&
                   _cur_args.len() == _lhs_args.len()
                {
                    match fill_pattern_mapping(_cur_args, _lhs_args, args_table) {
                        true => {},
                        false => { return false; }
                    }
                } else {
                    // current_expr: f(h(x))
                    // f(g(x, y)) => ..
                    return false;
                }
            }
        }
    }
    return true;
}

// Recursively traverses the right hand side expression to produce a new expression 
// with the corresponding symbols mapped using args_table 
fn construct_rhs(right: &Expr, args_table: &HashMap<Expr, Expr>) -> Result<Expr, Box<dyn Error>> {
    
    match right {
        Expr::Variable { iden, .. } => {
            // g(A)
            // g(x) => x
            if let Some(new_arg) = args_table.get(right) {
                Ok(new_arg.clone())
            } else {
                Ok(Expr::Variable { iden: iden.clone() })
            }
        },
        Expr::Functor { iden, args } => {
            // g(A)
            // g(x) => f(y, x)
            let mut new_args = vec![];
            for arg in args {
                let res = construct_rhs(arg, args_table)?;
                new_args.push(res);
            }

            Ok(Expr::Functor { 
                iden: iden.clone(), 
                args: new_args 
            })
        }
    }
}

fn find_binary_ops(expr: &Expr) -> bool {
    match expr {
        Expr::Variable { .. } => false,
        Expr::Functor { iden, args } => {
            if args.len() == 2 && Expr::get_binary_operator_str(iden.as_str()).is_some() {
                true
            } else {
                for arg in args.iter() {
                    if find_binary_ops(arg) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use crate::lexer::*;
    use crate::parser::Parser;
    use super::*;
    

    #[test]
    fn runtime_test() {
        let input_string = "
            f(A)
            f(x) => g(x, x) at 0
            g(x, x) => g(f(x), f(x)) at 0
        ";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);
        
        let mut parser = Parser::new();
        let _ = parser.parse(&mut lexer);

        let mut env = Env::new();
        let res = env.interpret(parser.stmts);

        assert!(res.is_ok());
        assert_eq!(env.get_expr(), 
                Some(&Expr::Functor { 
                    iden: "g".to_string(), 
                    args: vec![
                        Expr::Functor { iden: "f".to_string(), args: vec![Expr::Variable { iden: "A".to_string() }] },
                        Expr::Functor { iden: "f".to_string(), args: vec![Expr::Variable { iden: "A".to_string() }] }
                    ]
                })
        );
    }

    #[test]
    fn runtime_test_all_examples() -> Result<(), Box<dyn Error>>{
        
        let file_names = vec![
            "swap_pair.rx".to_string(),
            "peano.rx".to_string(),
            "simple_power_rule_calculus.rx".to_string(),
            "limit_power_rule_calculus.rx".to_string(),
        ];
        // let mut examples = vec![];
        let path = "examples/".to_string();
        for file_name in file_names {
            let file_name = path.to_owned() + &file_name;
            let input_string = fs::read_to_string(file_name)?;
            
            let mut lexer = Lexer::new();
            lexer.lex(input_string.as_str());
            
            let mut parser = Parser::new();
            let res = parser.parse(&mut lexer);
            assert!(res.is_ok());
            
            let mut env = Env::new();
            let res = env.interpret(parser.stmts);
    
            assert!(res.is_ok());
            // examples.push(env.)
        }
        Ok(())

        // assert_eq!(env.get_expr(), 
        //         Some(&Expr::Functor { 
        //             iden: "g".to_string(), 
        //             args: vec![
        //                 Expr::Functor { iden: "f".to_string(), args: vec![Expr::Variable { iden: "A".to_string() }] },
        //                 Expr::Functor { iden: "f".to_string(), args: vec![Expr::Variable { iden: "A".to_string() }] }
        //             ]
        //         })
        // );
    }
}