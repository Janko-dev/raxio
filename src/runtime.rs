use std::collections::HashMap;

use crate::parser::{Expr, Stmt};


pub struct Env {
    // current expression that is being manipulated
    pub current_expr: Option<Expr>,
    
    // true if in pattern matching state and false if in global state
    pub is_matching: bool,

    // rules hashmap from (string -> (Expr, Expr))
    pub rules: HashMap<String, (Expr, Expr)>
}

impl Env {
    pub fn new() -> Self {
        Self { 
            current_expr: None, 
            is_matching: false, 
            rules: HashMap::new() 
        }
    }

    pub fn print_prefix(&self) {
        if self.is_matching {
            print!("    ~ ");
        } else {
            print!("> ");
        }
    }

    fn print_current_expr(&self, prefix: &str) {
        match &self.current_expr {
            Some(expr) => { println!("{}{}", prefix, expr); },
            _ => {}
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {

        for stmt in stmts {
            match (stmt, self.is_matching) {
                (Stmt::ExprStmt(expr), false) => {
                    self.is_matching = true;
                    self.current_expr = Some(expr);
                    self.print_current_expr("Start matching: ");
                },
                (Stmt::ExprStmt(expr), true) => {
                    if let Expr::Variable { iden, depth: Some(d)} = expr {
                        if self.rules.contains_key(&iden) {
                            let (left, right) = self.rules.get(&iden).unwrap();
                            // pattern match
                            self.current_expr = Some(traverse_patterns(
                                self.current_expr.take().unwrap(), 
                                &left, 
                                &right,
                                d,
                            )?);
                            self.print_current_expr("    ");
                        }
                    } else {
                        return Err("Provide a valid and defined rule and a value for the depth to pattern match.".to_string());
                    }
                },
                (Stmt::DefineStmt { iden, left, right }, _) => {
                    self.rules.insert(iden, (left, right));
                },
                (Stmt::RuleStmt { left, right, depth}, true) => {
                    // inline rule, pattern match directly on current_expr
                    self.current_expr = Some(traverse_patterns(
                        self.current_expr.take().unwrap(), 
                        &left, 
                        &right,
                        depth,
                    )?);
                    self.print_current_expr("    ");
                },
                (Stmt::RuleStmt { .. }, false) => {/* possibly educate user about using inline rule outside of matching context */},
                (Stmt::EndStmt, true) => { 
                    self.print_current_expr("Result: ");
                    self.is_matching = false;
                },
                (Stmt::EndStmt, false) => { },
            }
        }
        Ok(())
    }

}

fn traverse_patterns(current_expr: Expr, left: &Expr, right: &Expr, depth: usize) -> Result<Expr, String>{

    if depth == 0 {
        // Only match against patterns for some depth
        let current_expr = match_patterns(current_expr, left, right)?;
        Ok(current_expr)
    } else {
        match current_expr {
            cur @ Expr::Variable { .. } => Ok(cur),
            Expr::Functor { iden, args } => {
                let mut new_args = vec![];
                for arg in args {
                    let expr = traverse_patterns(arg, left, right, depth - 1)?;
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
                Ok(Expr::Variable { iden: current, depth: None })
            }
        },
        (Expr::Functor { iden: current_iden, args: current_args },
         Expr::Functor { iden: lhs_iden, args: lhs_args }) => {
            if current_iden.as_str() == lhs_iden.as_str() &&
               current_args.len() == lhs_args.len()  
            {   
                let mut args_table = HashMap::<Expr, Expr>::new();
                for (lhs_arg, curr_arg) in lhs_args.iter().zip(current_args.iter())
                {
                    args_table.insert(lhs_arg.clone(), curr_arg.clone());
                }

                let res = construct_rhs(right, &args_table)?;
                Ok(res)
            } else {
                Ok(Expr::Functor { 
                    iden: current_iden, 
                    args: current_args 
                })
            }
        },
        // Cannot match variable against functor as the functor is a superset of the variable
        // i.e., contains more information.
        (cur @ Expr::Variable { .. }, Expr::Functor { .. }) => Ok(cur),
        (Expr::Functor { iden: current_iden, args: current_args }, 
        Expr::Variable { iden: lhs_iden, .. }) => {
            // f(x)
            // x => ..
            let mut new_args = vec![];
            for arg in current_args {
                if let Expr::Variable { iden, depth } = arg {
                    if iden.as_str() == lhs_iden.as_str() {
                        new_args.push(right.clone());
                    } else {
                        new_args.push(Expr::Variable { iden, depth });
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

fn construct_rhs(right: &Expr, args_table: &HashMap<Expr, Expr>) -> Result<Expr, String> {
    
    match right {
        Expr::Variable { iden, .. } => {
            // g(A)
            // g(x) => x
            if let Some(new_arg) = args_table.get(right) {
                Ok(new_arg.clone())
            } else {
                Ok(Expr::Variable { iden: iden.clone(), depth: None })
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

#[cfg(test)]
mod tests {
    use crate::lexer::*;
    use crate::parser::Parser;
    use super::*;
    

    // #[test]
    // fn runtime_test() {
    //     let input_string = "
    //         f(A)
    //         f(x) => g(x, x)
    //         g(x, x) => g(f(x), f(x))

    //     ";
    //     let mut lexer = Lexer::new();
    //     lexer.lex(input_string);
        
    //     let mut parser = Parser::new();
    //     let _ = parser.parse(&mut lexer);

    //     let mut env = Env::new();
    //     let res = env.interpret(parser.stmts);
    //     assert!(res.is_ok());
    //     assert_eq!(env.current_expr, 
    //             Some(Expr::Functor { 
    //                 iden: "g".to_string(), 
    //                 args: vec![
    //                     Expr::Functor { iden: "f".to_string(), args: vec![Expr::Variable { iden: "A".to_string() }] },
    //                     Expr::Functor { iden: "f".to_string(), args: vec![Expr::Variable { iden: "A".to_string() }] }
    //                 ]
    //             })
    //     );

    // }
}