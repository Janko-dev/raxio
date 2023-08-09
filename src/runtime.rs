use std::collections::HashMap;

use crate::parser::{Expr, Stmt};


pub struct Env {
    // History of all expressions after applying transformations.
    pub history: Vec<Expr>,
    
    // True if in pattern matching state and false if in global state
    pub is_matching: bool,

    // Rules hashmap from (string -> (lhs-expr, rhs-expr))
    pub rules: HashMap<String, (Expr, Expr)>
}

impl Env {
    pub fn new() -> Self {
        Self { 
            history: vec![],
            is_matching: false, 
            rules: HashMap::new() 
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
            self.print_current_expr("    ");
        }
    } 

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {

        // interpret each parsed statement.
        for stmt in stmts {
            // match on a statement and global/matching state.
            match (stmt, self.is_matching) {
                // If an expression is found, and we are not pattern matching
                // i.e., currently still in the global state, then start pattern matching.
                (Stmt::ExprStmt(expr), false) => {
                    self.is_matching = true;
                    self.history.push(expr);
                    self.print_current_expr("Start matching on: ");
                },
                (Stmt::ExprStmt(_), true) => {
                    // Has no effect, might be good to provide a hint
                },
                (Stmt::ApplyStmt { .. }, false) => {
                    // ERROR: Cannot apply rule outside of pattern matching context
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
                        self.print_current_expr("    ");
                    }
                },
                // Define statements can be constructed in either global or matching state.
                (Stmt::DefineStmt { iden, left, right }, _) => {
                    self.rules.insert(iden, (left, right));
                },
                // In-line rule statements are directlt mathed upon.
                (Stmt::RuleStmt { left, right, depth}, true) => {
                    self.history.push(ast_traverse_match(
                        self.get_expr().unwrap().clone(), 
                        &left, 
                        &right,
                        depth,
                    )?);
                    self.print_current_expr("    ");
                },
                // Currently, has no effect
                // TODO: Possibly educate user about using inline rule outside of matching context.
                (Stmt::RuleStmt { .. }, false) => {},
                (Stmt::EndStmt(s), true) => { 
                    self.print_current_expr("Result: ");
                    // write to file using s
                    self.history.clear();
                    self.is_matching = false;
                },
                (Stmt::EndStmt(_), false) => { },
            }
        }
        Ok(())
    }

}

// Traverse the Abstract Syntax Tree of the current expression, 
// and match sub-expression if and only if certain depth is reached.  
fn ast_traverse_match(current_expr: Expr, left: &Expr, right: &Expr, depth: usize) -> Result<Expr, String>{

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

fn match_patterns(current_expr: Expr, left: &Expr, right: &Expr) -> Result<Expr, String>{

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
            // Both functors have the same arity and the same identifier
            // then there are considered to produce the form of the right expr. 
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


fn construct_rhs(right: &Expr, args_table: &HashMap<Expr, Expr>) -> Result<Expr, String> {
    
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
            let _ = parser.parse(&mut lexer);
    
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