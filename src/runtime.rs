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

    fn print_current_expr(&self) {
        match &self.current_expr {
            Some(expr) => { println!("    {}", expr); },
            _ => {}
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {

        for stmt in stmts {
            match (stmt, self.is_matching) {
                (Stmt::ExprStmt(expr), false) => {
                    self.is_matching = true;
                    self.current_expr = Some(expr);
                },
                (Stmt::ExprStmt(expr), true) => {
                    if let Expr::Variable { iden } = expr {
                        if self.rules.contains_key(&iden) {
                            let (left, right) = self.rules.get(&iden).unwrap();
                            // pattern match
                            let depth = 1;
                            self.current_expr = Some(pattern_match(
                                self.current_expr.take().unwrap(), 
                                &left, 
                                &right, 
                                depth
                            )?);
                            self.print_current_expr();
                            
                        }
                    }
                },
                (Stmt::DefineStmt { iden, left, right }, _) => {
                    self.rules.insert(iden, (left, right));
                },
                (Stmt::RuleStmt { left, right }, true) => {
                    // inline rule, pattern match directly on current_expr
                    let depth = 1;
                    self.current_expr = Some(pattern_match(
                        self.current_expr.take().unwrap(), 
                        &left, 
                        &right, 
                        depth
                    )?);
                    self.print_current_expr();
                },
                (Stmt::RuleStmt { .. }, false) => {},
                (Stmt::EndStmt, _) => { self.is_matching = false; },
                _ => unreachable!()
            }
        }
        Ok(())
    }

}

fn pattern_match(current_expr: Expr, left: &Expr, right: &Expr, depth: usize) -> Result<Expr, String>{
    if depth < 1 {
        return Ok(current_expr);
    }

    match (current_expr, left) {
        (Expr::Variable { iden: current }, 
         Expr::Variable { iden: lhs }) => {
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

                match right {
                    Expr::Variable { iden } => {
                        Ok(Expr::Variable { iden: iden.clone() })
                    },
                    Expr::Functor { iden, args } => {
                        let mut args_table = HashMap::<Expr, Expr>::new();
                        for (lhs_arg, (curr_arg, rhs_arg)) in lhs_args.iter()
                                                                    .zip(current_args.iter().zip(args.iter()))
                        {
                            // args_table.insert(lhs_arg.clone(), curr_arg.clone());
                            args_table.insert(
                                lhs_arg.clone(),
                                pattern_match(curr_arg.clone(), lhs_arg, rhs_arg, depth-1)?
                            );
                        }

                        let mut new_args = vec![];
                        for arg in args {
                            if let Some(new_arg) = args_table.remove(arg) {
                                // f(g(A))
                                // f(g(x)) => g(f(x))
                                // g(f(A))

                                // g(g(A))
                                new_args.push(new_arg);
                            } else {
                                return Err(format!("Functor arguments of rhs do not match with arguments on lhs."));
                            }
                        }

                        Ok(Expr::Functor { 
                            iden: iden.clone(), 
                            args: new_args 
                        })
                    }
                }
            } else {
                Err("Functor expression did not match lhs rule.".to_string())
            }
        },
        (Expr::Variable { .. }, Expr::Functor { .. }) | 
        (Expr::Functor { .. }, Expr::Variable { .. }) => {
            Err("Cannot match variable expression with functor expression or vice versa.".to_string())
        },
        _ => unreachable!()

    }
}

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

