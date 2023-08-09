use std::{fmt::Display, error::Error};

use crate::{lexer::{Token, Lexer}, error::ParsingError};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Expr {
    Functor { iden: String, args: Vec<Expr> },
    Variable { iden: String, depth: Option<usize> },
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    RuleStmt {left: Expr, right: Expr, depth: usize},
    DefineStmt {iden: String, left: Expr, right: Expr}, 
    ExprStmt(Expr),
    EndStmt(Option<String>)
}

#[derive(Debug)]
pub struct Parser {
    pub stmts: Vec<Stmt>,
}

macro_rules! expect {
    ($expected_token:pat, $expected_str:expr, $lexer:expr) => {
        {
            let res = match $lexer.peek(0) {
                Some(tok) => {
                    if let $expected_token = *tok {
                        Ok(())
                    } else {
                        Err(Box::new(ParsingError::ExpectToken { 
                            expected: $expected_str, 
                            got: Some(tok.to_string()) 
                        }))
                    }
                },
                None => {
                    Err(Box::new(ParsingError::ExpectToken { 
                        expected: $expected_str, 
                        got: None 
                    }))
                }
            };
            res
        }
    };
    ($expected_token:path, $lexer:expr) => {
        {
            let res = match $lexer.peek(0) {
                Some(tok) => {
                    if let $expected_token = *tok {
                        Ok(())
                    } else {
                        Err(Box::new(ParsingError::ExpectToken { 
                            expected: $expected_token.to_string(), 
                            got: Some(tok.to_string()) 
                        }))
                    }
                },
                None => {
                    Err(Box::new(ParsingError::ExpectToken { 
                        expected: $expected_token.to_string(),
                        got: None
                    }))
                }
            };
            res
        }
    };
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Variable { iden, depth: Some(n) } => write!(f, "{} depth {}", iden, n),
            Expr::Variable { iden, depth: None } => write!(f, "{}", iden),
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

impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Variable { iden, depth: Some(n) } => format!("{} depth {}", iden, n),
            Expr::Variable { iden, depth: None } => format!("{}", iden),
            Expr::Functor { iden, args }  => {
                let mut res = String::new();
                if let (Some(op), 2) = (Self::get_binary_operator_str(iden.as_str()), args.len()) {
                    res.push_str(&format!("{} {} {}", &args[0].to_string(), op, &args[1].to_string()));
                } else {
                    if iden.as_str() == "group" {
                        res.push('(');
                    } else {
                        res.push_str(&format!("{}(", iden));
                    }
    
                    for (i, arg) in args.iter().enumerate() {
                        res.push_str(&arg.to_string());
                        if i < args.len() - 1 {
                            res.push_str(&", ");
                        }
                    }
                    res.push(')');
                }
                
                res
            }
        }
    }

    pub fn get_binary_operator_str(iden: &str) -> Option<&str> {
        match iden {
            "add" => Some("+"),
            "sub" => Some("-"),
            "mul" => Some("*"),
            "div" => Some("/"),
            _ => None
        }
    }
}

impl Parser{
    pub fn new() -> Self {
        Self { stmts: vec![] }
    }

    pub fn parse(&mut self, lexer: &mut Lexer) -> Result<(), Box<dyn Error>>{
        lexer.reset_iter();
        
        while !lexer.is_at_end() {
            match lexer.peek(0) {
                Some(Token::Define) => { self.parse_definition(lexer)?; },
                Some(Token::End) => { self.parse_end_stmt(lexer)?; },
                Some(_) => { self.parse_rule(lexer)?; },
                _ => unreachable!()
            }
        }
        Ok(())
    }

    fn parse_end_stmt(&mut self, lexer: &mut Lexer) -> Result<(), Box<dyn Error>> {
        
        lexer.next();
        let path = if let Some(Token::Path(s)) = lexer.peek(0) {
            let res = Some(s.to_owned());
            lexer.next();
            res
        } else {
            None
        };
        self.stmts.push(Stmt::EndStmt(path));
        Ok(())
    }

    fn parse_definition(&mut self, lexer: &mut Lexer) -> Result<(), Box<dyn Error>>{
        lexer.next();
        let iden = match lexer.peek(0) {
            Some(Token::Identifier(s)) => s.as_str().to_owned(),
            Some(tok) => return Err(
                Box::new(ParsingError::ExpectTokenAfter { 
                    expected: "identifier".to_string(), 
                    after: Token::Define.to_string(), 
                    got: Some(tok.to_string()) 
                })),
            None => return Err(Box::new(ParsingError::ExpectTokenAfter { 
                expected: "identifier".to_string(), 
                after: Token::Define.to_string(), 
                got: None
            }))
        };
        lexer.next();
        expect!(Token::As, lexer)?;
        lexer.next();

        let left= self.parse_term(lexer)?;
        expect!(Token::Derive, lexer)?;
        lexer.next();
        let right= self.parse_term(lexer)?;
        
        self.stmts.push(Stmt::DefineStmt { 
            iden, 
            left, 
            right
        });

        Ok(())
    }

    fn parse_rule(&mut self, lexer: &mut Lexer) -> Result<(), Box<dyn Error>> {
        
        let left = self.parse_term(lexer)?;
        if let Some(Token::Derive) = lexer.peek(0) {
            lexer.next();
            let right = self.parse_term(lexer)?;
            expect!(Token::At, lexer)?;
            lexer.next();
            if let Some(Token::Number(n)) = lexer.peek(0){
                self.stmts.push(Stmt::RuleStmt {
                    left, 
                    right,
                    depth: *n 
                });
                lexer.next();
                Ok(())
            } else {
                Err(Box::new(ParsingError::ExpectDepthValue))
            }
        } else {
            self.stmts.push(Stmt::ExprStmt(left));
            Ok(())
        }
    }

    fn parse_term(&mut self, lexer: &mut Lexer) -> Result<Expr, Box<dyn Error>> {
        let mut left = self.parse_factor(lexer)?;

        while let Some(Token::Add) | Some(Token::Sub) = lexer.peek(0) {
            let op = lexer.next().unwrap().clone();
            let right = self.parse_factor(lexer)?;
            left = Expr::Functor{
                iden: op.to_string(),
                args: vec![left, right]
            };    
        } 
        Ok(left)
    }

    fn parse_factor(&mut self, lexer: &mut Lexer) -> Result<Expr, Box<dyn Error>> {
        let mut left = self.parse_expr(lexer)?;

        while let Some(Token::Mul) | Some(Token::Div) = lexer.peek(0) {
            let op = lexer.next().unwrap().clone();
            let right = self.parse_expr(lexer)?;
            left = Expr::Functor{
                iden: op.to_string(),
                args: vec![left, right]
            };    
        } 
        Ok(left)
    }

    fn parse_expr(&mut self, lexer: &mut Lexer) -> Result<Expr, Box<dyn Error>> {

        match lexer.peek(0) {
            Some(Token::OpenParen) => {
                // group
                let args = self.parse_functor_args(lexer)?;
                Ok(Expr::Functor { 
                    iden: "group".to_string(), 
                    args
                })
            },
            Some(Token::Identifier(s)) => {
                let iden = s.to_owned();
                lexer.next();
                if let Some(Token::OpenParen) = lexer.peek(0) {
                    
                    let args = self.parse_functor_args(lexer)?;
                    Ok(Expr::Functor { 
                        iden, 
                        args
                    })
                } else {
                    let depth = if let Some(Token::At) = lexer.peek(0) {
                        lexer.next();
                        match lexer.peek(0) {
                            Some(Token::Number(n)) => {
                                let depth = Some(*n);
                                lexer.next();
                                depth
                            },
                            Some(tok) => {
                                return Err(Box::new(ParsingError::ExpectTokenAfter { 
                                    expected: "number".to_string(), 
                                    after: Token::At.to_string(), 
                                    got: Some(tok.to_string())
                                }));
                            },
                            None => {
                                return Err(Box::new(ParsingError::ExpectTokenAfter { 
                                    expected: "number".to_string(), 
                                    after: Token::At.to_string(), 
                                    got: None
                                }));
                            }
                        }
                    } else {
                        None
                    };
                    Ok(Expr::Variable { iden, depth })
                }
            },
            Some(Token::Number(n)) => {
                let res = Ok(Expr::Variable { iden: n.to_string(), depth: None });
                lexer.next();
                res
            }
            Some(tok) => Err(Box::new(ParsingError::UnexpectedToken { 
                got: Some(tok.to_string()) 
            })),
            None => Err(Box::new(ParsingError::UnexpectedToken { 
                got: None
            }))
        }
    }

    fn parse_functor_args(&mut self, lexer: &mut Lexer) -> Result<Vec<Expr>, Box<dyn Error>> {
        lexer.next();
        let mut args = vec![];
        loop {
            match lexer.peek(0) {
                Some(Token::CloseParen) => {
                    lexer.next();
                    break;
                },
                _ => {
                    args.push(self.parse_term(lexer)?);
                    if let Some(Token::Comma) = lexer.peek(0) {
                        lexer.next();
                    }
                }
            }
        };
        Ok(args)
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

        dbg!(&lexer.tokens);

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
                            Expr::Variable { iden: "p".to_string(), depth: None },
                            Expr::Variable { iden: "q".to_string(), depth: None }
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
                            Expr::Variable { iden: "p".to_string(), depth: None }
                        ]
                    },
                    Expr::Functor {
                        iden: "neg".to_string(), 
                        args: vec![
                            Expr::Variable { iden: "q".to_string(), depth: None }
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
        let input_string = "f(x, y, z) def x as x(z) => z(x) end \"hello world\"";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);
        
        dbg!(&res);
        assert!(!res.is_err());
        assert!(parser.stmts.len() == 3);

        assert_eq!(
            parser.stmts[0], 
            Stmt::ExprStmt(Expr::Functor { 
                iden: "f".to_string(), 
                args: vec![
                    Expr::Variable { iden: "x".to_string(), depth: None },
                    Expr::Variable { iden: "y".to_string(), depth: None },
                    Expr::Variable { iden: "z".to_string(), depth: None }
                ]
            })
        );
        assert_eq!(
            parser.stmts[1], 
            Stmt::DefineStmt { 
                iden: "x".to_string(), 
                left:  Expr::Functor { iden: "x".to_string(), args: vec![Expr::Variable { iden: "z".to_string(), depth: None }] }, 
                right: Expr::Functor { iden: "z".to_string(), args: vec![Expr::Variable { iden: "x".to_string(), depth: None }] }, 
            }
        );

        assert_eq!(
            parser.stmts[2], 
            Stmt::EndStmt(Some("hello world".to_string()))
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