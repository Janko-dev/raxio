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
                            self.current_expr = Some(pattern_match(
                                self.current_expr.take().unwrap(), 
                                &left, 
                                &right,
                                d,
                            )?);
                            self.print_current_expr("    ");
                            
                        }
                    } else {
                        return Err("provide a value for the depth to traverse the matching expression.".to_string());
                    }
                },
                (Stmt::DefineStmt { iden, left, right }, _) => {
                    self.rules.insert(iden, (left, right));
                },
                (Stmt::RuleStmt { left, right, depth}, true) => {
                    // inline rule, pattern match directly on current_expr
                    self.current_expr = Some(pattern_match(
                        self.current_expr.take().unwrap(), 
                        &left, 
                        &right,
                        depth,
                    )?);
                    self.print_current_expr("    ");
                },
                (Stmt::RuleStmt { .. }, false) => {},
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

fn pattern_match(current_expr: Expr, left: &Expr, right: &Expr, depth: usize) -> Result<Expr, String>{
    _pattern_match(current_expr, left, right, depth, 0)
}

fn _pattern_match(current_expr: Expr, left: &Expr, right: &Expr, depth: usize, acc: usize) -> Result<Expr, String>{

    if acc == depth {
        // println!("MATCH THIS");
        let current_expr = do_matching(current_expr, left, right)?;
        Ok(current_expr)
    } else {
        match current_expr {
            cur @ Expr::Variable { .. } => Ok(cur),
            Expr::Functor { iden, args } => {
                let mut new_args = vec![];
                for arg in args {
                    let expr = _pattern_match(arg, left, right, depth, acc + 1)?;
                    // println!("{}", expr);
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

fn do_matching(current_expr: Expr, left: &Expr, right: &Expr) -> Result<Expr, String>{

    match (current_expr, left) {
        (Expr::Variable { iden: current , ..}, 
         Expr::Variable { iden: lhs, .. }) => {
            if current.as_str() == lhs.as_str() {
                Ok(right.clone())
            } else {
                Err("Variable expression did not match lhs rule.".to_string())
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
                // Err("Functor expression did not match left hand side of rule.".to_string())
                Ok(Expr::Functor { 
                    iden: current_iden, 
                    args: current_args 
                })
            }
        },

        // f(x)
        // x => 
        
        (cur @ Expr::Variable { .. }, Expr::Functor { .. }) |
        (cur @ Expr::Functor { .. }, Expr::Variable { .. }) => {
            // println!("Cannot match variable expression with functor expression or vice versa.");
            Ok(cur)
            // Err("Cannot match variable expression with functor expression or vice versa.".to_string())
        },
        // (Expr::Functor { iden: current_iden, args: current_args }, 
        //  Expr::Variable { iden: lhs_iden, .. }) => {}
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
                // mul(n, pow(x, sub(n, 1)))
                // if let Some(new_arg) = args_table.get(arg) {
                //     // f(g(A))
                //     // f(g(x)) => g(f(x))
                //     // g(f(A))

                //     // g(g(A))
                    
                //     println!("TEST, {:?}", right);
                //     new_args.push(res);
                // } else {
                //     new_args.push(arg.clone());
                //     // return Err(format!("Functor arguments of rhs do not match with arguments on lhs."));
                // }
            }

            Ok(Expr::Functor { 
                iden: iden.clone(), 
                args: new_args 
            })
        }
    }
}

// fn pattern_match(current_expr: Expr, left: &Expr, right: &Expr, depth: usize) -> Result<Expr, String>{
//     if depth < 1 {
//         return Ok(current_expr);
//     }

//     match (current_expr, left) {
//         (Expr::Variable { iden: current }, 
//          Expr::Variable { iden: lhs }) => {
//             if current.as_str() == lhs.as_str() {
//                 Ok(right.clone())
//             } else {
//                 Err("Variable expression did not match lhs rule.".to_string())
//             }
//         },
//         (Expr::Functor { iden: current_iden, args: current_args },
//          Expr::Functor { iden: lhs_iden, args: lhs_args }) => {
//             if current_iden.as_str() == lhs_iden.as_str() &&
//                current_args.len() == lhs_args.len()  
//             {

//                 match right {
//                     Expr::Variable { iden } => {
//                         Ok(Expr::Variable { iden: iden.clone() })
//                     },
//                     Expr::Functor { iden, args } => {
//                         let mut args_table = HashMap::<Expr, Expr>::new();
//                         for (lhs_arg, curr_arg) in lhs_args.iter().zip(current_args.iter())
//                         {
//                             // args_table.insert(lhs_arg.clone(), curr_arg.clone());
//                             args_table.insert(
//                                 lhs_arg.clone(),
//                                 pattern_match(curr_arg.clone(), left, right, depth-1)?
//                             );
//                         }

//                         let mut new_args = vec![];
//                         for arg in args {
//                             if let Some(new_arg) = args_table.get(arg) {
//                                 // f(g(A))
//                                 // f(g(x)) => g(f(x))
//                                 // g(f(A))

//                                 // g(g(A))
//                                 new_args.push(new_arg.clone());
//                             } else {
//                                 return Err(format!("Functor arguments of rhs do not match with arguments on lhs."));
//                             }
//                         }

//                         Ok(Expr::Functor { 
//                             iden: iden.clone(), 
//                             args: new_args 
//                         })
//                     }
//                 }
//             } else {
//                 Err("Functor expression did not match lhs rule.".to_string())
//             }
//         },
//         (Expr::Variable { .. }, Expr::Functor { .. }) | 
//         (Expr::Functor { .. }, Expr::Variable { .. }) => {
//             Err("Cannot match variable expression with functor expression or vice versa.".to_string())
//         },
//         // _ => unreachable!()

//     }
// }

// > def switch as f(a, b) => f(b, a)
// > f(1, 0)
//   ~ switch 
//     $ f(0, 1)
//   ~ f(a, b) => g(f(a), f(b))
//     $ g(f(0), f(1))
//   ~ switch
//     $ g(f(1), f(0))
//   ~ end
// > 

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