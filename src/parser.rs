use std::fmt::Display;

use crate::lexer::{Token, Lexer};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    Functor { iden: String, args: Vec<Expr> },
    Variable { iden: String },
    // Constant { num: f32 },
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    RuleStmt {left: Expr, right: Expr}, 
    DefineStmt {iden: String, left: Expr, right: Expr}, 
    ExprStmt(Expr),
    EndStmt
}

#[derive(Debug)]
pub struct Parser {
    pub stmts: Vec<Stmt>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Variable { iden } => write!(f, "{}", iden),
            Expr::Functor { iden, args }  => {
                write!(f, "{}(", iden)?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;

                Ok(())
            }
        }
    }
}

impl Parser{
    pub fn new() -> Self {
        Self { stmts: vec![] }
    }

    pub fn parse(&mut self, lexer: &mut Lexer) -> Result<(), String>{
        lexer.reset_iter();
        
        while !lexer.is_at_end() {
            match lexer.peek(0) {
                Some(Token::Define) => { self.parse_definition(lexer)?; },
                Some(Token::End) => { self.stmts.push(Stmt::EndStmt); lexer.next(); },
                Some(_) => { self.parse_rule(lexer)?; },
                _ => unreachable!()
            }
        }
        Ok(())
    }

    fn parse_definition(&mut self, lexer: &mut Lexer) -> Result<(), String>{
        lexer.next();
        let iden = match lexer.peek(0) {
            Some(Token::Identifier(s)) => s.as_str().to_owned(),
            Some(tok) => return Err(format!("Expected identifier after 'def', but got {:?}.", tok)),
            None => return Err(format!("Expected identifier after 'def', but got nothing."))
        };
        lexer.next();
        self.expect(Token::As, lexer)?;

        let left= self.parse_expr(lexer)?;
        self.expect(Token::Derive, lexer)?;
        let right= self.parse_expr(lexer)?;
        
        self.stmts.push(Stmt::DefineStmt { 
            iden, 
            left, 
            right
        });

        Ok(())
    }

    fn expect(&mut self, expected_token: Token, lexer: &mut Lexer) -> Result<(), String> {
        match lexer.peek(0) {
            Some(tok) => {
                if *tok == expected_token {
                    lexer.next();
                    Ok(())
                } else {
                    Err(format!("Expected {:?}, but got {:?}.", expected_token, tok))
                }
            },
            None => Err(format!("Expected {:?}, but got nothing.", expected_token))
        }
    }

    fn parse_rule(&mut self, lexer: &mut Lexer) -> Result<(), String> {
        
        let left = self.parse_expr(lexer)?;
        if let Some(Token::Derive) = lexer.peek(0) {
            lexer.next();
            let right = self.parse_expr(lexer)?;
            self.stmts.push(Stmt::RuleStmt {
                left, 
                right 
            });
            Ok(())
        } else {
            self.stmts.push(Stmt::ExprStmt(left));
            Ok(())
        }
    }

    fn parse_expr(&mut self, lexer: &mut Lexer) -> Result<Expr, String> {

        match lexer.peek(0) {
            Some(Token::Identifier(s)) => {
                let iden = s.to_owned();
                lexer.next();
                if let Some(Token::OpenParen) = lexer.peek(0) {
                    lexer.next();
                    let mut args = vec![];
                    loop {
                        match lexer.peek(0) {
                            Some(Token::CloseParen) => {
                                lexer.next();
                                break;
                            },
                            _ => {
                                args.push(self.parse_expr(lexer)?);
                                if let Some(Token::Comma) = lexer.peek(0) {
                                    lexer.next();
                                }
                            }
                        }
                    };
        
                    Ok(Expr::Functor { 
                        iden, 
                        args
                    })
                } else {
                    Ok(Expr::Variable { iden })
                }
            },
            // Some(Token::Number(num)) => {
            //     let num = num.to_owned();
            //     lexer.next();
            //     Ok(Expr::Constant { num })
            // },
            Some(tok) => return Err(format!("Expected constant or variable, but got {:?}.", tok)),
            None => return Err(format!("Expected constant or variable, but got nothing."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_demorgan() {
        let input_string = "def demorgan as neg(or(p, q)) => and(neg(p), neg(q))";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);

        assert!(!res.is_err());
        assert!(parser.stmts.len() == 1);

        let test_stmt = Stmt::DefineStmt { 
            iden: "demorgan".to_string(), 
            left: Expr::Functor { 
                iden: "neg".to_string(), 
                args: vec![
                    Expr::Functor {
                        iden: "or".to_string(), 
                        args: vec![
                            Expr::Variable { iden: "p".to_string() },
                            Expr::Variable { iden: "q".to_string() }
                        ]
                    }
                ] 
            }, 
            right: Expr::Functor { 
                iden: "and".to_string(), 
                args: vec![
                    Expr::Functor {
                        iden: "neg".to_string(), 
                        args: vec![
                            Expr::Variable { iden: "p".to_string() }
                        ]
                    },
                    Expr::Functor {
                        iden: "neg".to_string(), 
                        args: vec![
                            Expr::Variable { iden: "q".to_string() }
                        ]
                    },
                ] 
            }, 
        };

        let parsed_stmt = parser.stmts.swap_remove(0);
        assert_eq!(parsed_stmt, test_stmt);
    }

    #[test] 
    fn parse_multiple_stmts() {
        let input_string = "f(x, y, z) def x as x(z) => z(x) end";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);

        assert!(!res.is_err());
        assert!(parser.stmts.len() == 3);

        assert_eq!(
            parser.stmts[0], 
            Stmt::ExprStmt(Expr::Functor { 
                iden: "f".to_string(), 
                args: vec![
                    Expr::Variable { iden: "x".to_string() },
                    Expr::Variable { iden: "y".to_string() },
                    Expr::Variable { iden: "z".to_string() }
                ]
            })
        );
        assert_eq!(
            parser.stmts[1], 
            Stmt::DefineStmt { 
                iden: "x".to_string(), 
                left:  Expr::Functor { iden: "x".to_string(), args: vec![Expr::Variable { iden: "z".to_string() }] }, 
                right: Expr::Functor { iden: "z".to_string(), args: vec![Expr::Variable { iden: "x".to_string() }] }, 
            }
        );

        assert_eq!(
            parser.stmts[2], 
            Stmt::EndStmt
        );
    }

    #[test]
    fn trigger_definition_error() {
        let input_string = "def x x(z) => z(x)";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);

        assert!(res.is_err());
    }

    #[test]
    fn trigger_rule_error() {
        let input_string = "def f as x(z) = z(x)";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);

        assert!(res.is_err());
    }
}