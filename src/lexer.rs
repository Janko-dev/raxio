
use std::error::Error;

use super::error::LexError;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String)  , // alphabetic identifier
    Number(usize)       , // unsigned integer
    Path(String)        , // "/path/to/file"

    OpenParen   , // (
    CloseParen  , // )
    Comma       , // ,
    Derive      , // =>


    Define      , // def
    As          , // as
    End         , // end
    At          , // at

    Add         , // +
    Sub         , // -
    Mul         , // *
    Div         , // /
}

const KEY_DEF: &str = "def";
const KEY_END: &str = "end";

#[derive(Debug)]
pub struct Lexer{
    pub tokens: Vec<Token>,
    pub errors: Vec<Box<dyn Error>>,
    pub idx: usize
}

type PeekIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

impl Token {
    pub fn to_string(&self) -> String {
        match self {
            Token::Add => "add".to_string(),
            Token::Sub => "sub".to_string(),
            Token::Mul => "mul".to_string(),
            Token::Div => "div".to_string(),
            Token::Identifier(s) => format!("identifier literal '{}'", s),
            Token::Number(n) => format!("number literal '{}'", n),
            Token::Path(s) => format!("path literal '{}'", s),
            Token::OpenParen => "open parenthesis ('(')".to_string(),   
            Token::CloseParen => "closed parenthesis (')')".to_string(),  
            Token::Comma => "comma (',')".to_string(),       
            Token::Derive => "derive symbol ('=>')".to_string(),      
            Token::Define => "define-keywork ('def')".to_string(),      
            Token::As => "as-keyword ('as')".to_string(),          
            Token::End => "end-keyword ('end')".to_string(),         
            Token::At => "at-keyword ('at')".to_string(),    
        }
    }
}

impl Lexer {
    
    pub fn new() -> Self {
        Self { tokens: vec![], errors: vec![], idx: 0 }
    }

    fn push_token(&mut self, token: Token, input_bytes: &mut PeekIter) {
        self.tokens.push(token);
        input_bytes.next();
    }

    fn push_keyword(&mut self, 
        token: Token, 
        keyword: &str, 
        input_bytes: &mut PeekIter, 
        current_idx: usize, 
        input_string: &str) 
    {
        let mut count = 0;

        let res = input_string
            .chars()
            .skip(current_idx)
            .zip(keyword.chars())
            .inspect(|_| count +=1 )
            .map(|(x, y)| x == y)
            .reduce(|acc, b| acc & b)
            .is_some_and(|x| x == true);

        if res && count == keyword.len() {
            let next_char = input_string
                .chars()
                .skip(current_idx + count)
                .next();

            if let Some(' ') | Some('\n') |
                   Some('\t') | Some('\r') |
                   None = next_char 
            {
                for _ in 0..keyword.len() {
                    input_bytes.next();
                }
                self.tokens.push(token);
            } else {
                self.push_identifier(input_bytes);
            }

        } else {
            // possibly identifier
            self.push_identifier(input_bytes);
        }
    }

    fn push_identifier(&mut self, input_bytes: &mut PeekIter) {
        let mut lexeme = String::new();
        while let Some((_, c @ 'a'..='z')) |
                  Some((_, c @ 'A'..='Z')) |
                  Some((_, c @ '_')) | 
                  Some((_, c @ '0'..='9')) = input_bytes.peek() 
        {
            lexeme.push(*c);
            input_bytes.next();
        }

        self.tokens.push(Token::Identifier(lexeme));
    }

    fn push_number(&mut self, input_bytes: &mut PeekIter) {
        let mut collected_digits = String::new();
        while let Some((_, d @ '0'..='9')) = input_bytes.peek() {
            collected_digits.push(*d);
            input_bytes.next();
        }
        
        match collected_digits.parse::<usize>() {
            Ok(n) => self.tokens.push(Token::Number(n)),
            Err(msg) => self.errors.push(Box::new(msg))
        }

        // match input_bytes.peek() {
        //     Some((_, ' ')) | Some((_, '\n')) |
        //     Some((_, '\t')) | Some((_, '\r')) | 
        //     None => { input_bytes.next(); },
        //     Some((i, c)) => { self.errors.push(format!("Expected whitespace or number, but found '{}' at position {} during lexing.", *c, *i)); }
        // }
    }

    fn push_path(&mut self, input_bytes: &mut PeekIter) {
        input_bytes.next();
        let mut lexeme = String::new();
        if let Some((_, '/')) = input_bytes.peek() {
            lexeme.push('/');
            input_bytes.next();
        }
        while let Some((_, c @ 'a'..='z')) |
                  Some((_, c @ 'A'..='Z')) |
                  Some((_, c @ '_')) | 
                  Some((_, c @ '0'..='9')) | 
                  Some((_, c @ ' ')) | Some((_, c @ '\n')) |
                  Some((_, c @ '\t')) | Some((_, c @ '\r')) = input_bytes.peek() 
        {
            lexeme.push(*c);
            input_bytes.next();
        }

        
        while let Some((_, '/')) = input_bytes.peek() {
            lexeme.push('/');
            input_bytes.next();
            while let Some((_, c @ 'a'..='z')) |
                    Some((_, c @ 'A'..='Z')) |
                    Some((_, c @ '_')) | 
                    Some((_, c @ '0'..='9')) |
                    Some((_, c @ ' ')) | Some((_, c @ '\n')) |
                    Some((_, c @ '\t')) | Some((_, c @ '\r')) = input_bytes.peek() 
            {
                lexeme.push(*c);
                input_bytes.next();
            }
        }

        match input_bytes.peek() {
            Some((_, '"')) => { self.tokens.push(Token::Path(lexeme)); },
            Some((i, _)) => { self.errors.push(Box::new(LexError::UnterminatedStringLiteral { pos: *i })); },
            None => { self.errors.push(Box::new(LexError::UnterminatedStringLiteralAtEnd)); }
        }
        input_bytes.next();

    }

    pub fn lex<'a>(&mut self, input_string: &'a str) {
        let mut input_bytes: PeekIter = input_string.char_indices().peekable();

        while input_bytes.peek().is_some() {

            match input_bytes.peek() {
                Some((_, ',')) => { self.push_token(Token::Comma,      &mut input_bytes); },
                Some((_, '(')) => { self.push_token(Token::OpenParen,  &mut input_bytes); },
                Some((_, ')')) => { self.push_token(Token::CloseParen, &mut input_bytes); },
                Some((_, '+')) => { self.push_token(Token::Add, &mut input_bytes); },
                Some((_, '-')) => { self.push_token(Token::Sub, &mut input_bytes); },
                Some((_, '*')) => { self.push_token(Token::Mul, &mut input_bytes); },
                Some((_, '/')) => { self.push_token(Token::Div, &mut input_bytes); },
                Some((_, '"')) => { self.push_path(&mut input_bytes); },
                Some((_, '=')) => {
                    input_bytes.next();
                    match input_bytes.peek() {
                        Some((_, '>')) => {
                            self.push_token(Token::Derive, &mut input_bytes);
                        },
                        Some((i, c)) => {
                            self.errors.push(Box::new(LexError::ExpectCharAfter {
                                pos: *i, 
                                expected: '>', 
                                after: '=', 
                                got: *c 
                            }));
                            input_bytes.next();
                        },
                        None => {
                            self.errors.push(Box::new(LexError::ExpectCharAfter {
                                pos: input_string.len()-1, 
                                expected: '>', 
                                after: '=', 
                                got: ' ' 
                            }));
                            input_bytes.next();
                        }
                    }
                },
                Some((_, ' ')) | Some((_, '\t')) | 
                Some((_, '\r')) | Some((_, '\n')) => { input_bytes.next(); },
                Some((i, 'd')) => {
                    let current_idx = *i;
                    self.push_keyword(Token::Define, KEY_DEF, &mut input_bytes, current_idx, input_string); 
                },
                Some((i, 'e')) => {
                    let current_idx = *i;
                    self.push_keyword(Token::End, KEY_END, &mut input_bytes, current_idx, input_string); 
                },
                Some((i, 'a')) => {
                    match input_string.chars().nth(*i + 1) {
                        Some('s') => { self.push_token(Token::As, &mut input_bytes); input_bytes.next(); },
                        Some('t') => { self.push_token(Token::At, &mut input_bytes); input_bytes.next(); },
                        Some(_) => { self.push_identifier(&mut input_bytes); },
                        None => { input_bytes.next(); }
                    } 
                },
                Some((_, 'a'..='z')) | Some((_, 'A'..='Z')) | Some((_, '_'))=> {
                    self.push_identifier(&mut input_bytes);
                },
                Some((_, '0'..='9')) => {
                    self.push_number(&mut input_bytes);
                },
                Some((i, c)) => {
                    self.errors.push(Box::new(LexError::UnknownChar { 
                        pos: *i, 
                        got: *c 
                    })); 
                    input_bytes.next();
                }
                _ => {unreachable!()}
            }        
        } 
    }

    pub fn reset_iter(&mut self) {
        self.idx = 0;
    }

    pub fn peek(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.idx + n)
    }

    pub fn next(&mut self) -> Option<&Token> {
        self.idx += 1;
        self.tokens.get(self.idx-1)
    }

    pub fn is_at_end(&self) -> bool {
        self.tokens.get(self.idx).is_none()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_definition() {
        let input_string = "def pair as f(x, y) => f(y, x)";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let iter = lexer.tokens.iter();
        let test = vec![
            Token::Define,
            Token::Identifier("pair".to_string()),
            Token::As,
            Token::Identifier("f".to_string()),
            Token::OpenParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::CloseParen,
            Token::Derive,
            Token::Identifier("f".to_string()),
            Token::OpenParen,
            Token::Identifier("y".to_string()),
            Token::Comma,
            Token::Identifier("x".to_string()),
            Token::CloseParen
        ];
        assert!(iter.eq(test.iter()));
        
    }

    #[test]
    fn lex_keyword_combinations() {
        let input_string = "defas enddef as";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        let iter = lexer.tokens.iter();
        let test = vec![
            Token::Identifier("defas".to_string()),
            Token::Identifier("enddef".to_string()),
            Token::As,
        ];
        assert!(iter.eq(test.iter()));
    }

    #[test]
    fn trigger_equal_sign_and_numeric_error() {
        let input_string = " a = a + a";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);

        dbg!(&lexer.errors);
        
        assert!(lexer.errors.len() == 1);
        let e = lexer.errors.swap_remove(0);
        assert!(e.is::<LexError>());
        assert_eq!(
            *e.downcast::<LexError>().unwrap().clone(), 
            LexError::ExpectCharAfter { 
                pos: 4, 
                expected: '>', 
                after: '=', 
                got: ' ' 
            }
        );
    }

    #[test]
    fn trigger_unterminated_string_literal_error() {
        let input_string = "abc \"path ";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);
        
        assert!(lexer.errors.len() == 1);
        let e = lexer.errors.swap_remove(0);
        assert!(e.is::<LexError>());
        assert_eq!(
            *e.downcast::<LexError>().unwrap().clone(), 
            LexError::UnterminatedStringLiteralAtEnd
        );
    }

    #[test]
    fn lex_infix_math_ops() {
        let input_string = "(5 + 6) * 3-1";
        let mut lexer = Lexer::new();
        lexer.lex(input_string);
        
        let iter = lexer.tokens.iter();
        let test = vec![
            Token::OpenParen,
            Token::Number(5),
            Token::Add,
            Token::Number(6),
            Token::CloseParen,
            Token::Mul,
            Token::Number(3),
            Token::Sub,
            Token::Number(1),
        ];
        assert!(iter.eq(test.iter()));
    }
}
