

#[derive(Debug)]
pub enum Token {
    Number(f32)      , // number, float 
    Identifier(String)  , // alphabetic identifier

    OpenParen   , // (
    CloseParen  , // )
    Comma       , // ,
    Derive      , // =>

    Define      , // def
    As          , // as
    End         , // end
}

const KEY_DEF: &str = "def";
const KEY_AS: &str  = "as";
const KEY_END: &str = "end";

#[derive(Debug)]
pub struct TokenList{
    pub tokens: Vec<Token>,
    pub errors: Vec<String>
}

type PeekIter<'a> = std::iter::Peekable<std::str::CharIndices<'a>>;

impl TokenList {
    
    pub fn new() -> Self {
        Self { tokens: vec![], errors: vec![] }
    }

    fn push_token(&mut self, token: Token, input_bytes: &mut PeekIter) {
        self.tokens.push(Token::Comma);
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
            for _ in 0..keyword.len() {
                input_bytes.next();
            }
            self.tokens.push(token);
        } else {
            // possibly identifier
        }
    }

    fn push_number(&mut self, input_bytes: &mut PeekIter) {
        let mut collected_digits = String::new();
        while let Some((_, d @ '0'..='9')) = input_bytes.peek() {
            collected_digits.push(*d);
            input_bytes.next();
        }

        if let Some ((_, '.')) = input_bytes.peek() {
            input_bytes.next();
            collected_digits.push('.' as char);
            while let Some((_, d @ '0'..='9')) = input_bytes.next() {
                collected_digits.push(d);
            }
        }

        let res = collected_digits.parse::<f32>();

        match res {
            Ok(n) => self.tokens.push(Token::Number(n)),
            Err(msg) => self.errors.push(msg.to_string())
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

    pub fn lex<'a>(&mut self, input_string: &'a str) {
        let mut input_bytes: PeekIter = input_string.char_indices().peekable();

        while input_bytes.peek().is_some() {

            match input_bytes.peek() {
                Some((_, ',')) => { self.push_token(Token::Comma,      &mut input_bytes) },
                Some((_, '(')) => { self.push_token(Token::OpenParen,  &mut input_bytes) },
                Some((_, ')')) => { self.push_token(Token::CloseParen, &mut input_bytes) },
                Some((_, '=')) => {
                    input_bytes.next();
                    if let Some((_, '>')) = input_bytes.peek() {
                        self.tokens.push(Token::Derive);
                        input_bytes.next();
                    } else {
                        self.errors.push("Expected '>' after '=' during lexing.".to_string());
                    }
                },
                Some((_, ' ')) | Some((_, '\t')) | 
                Some((_, '\r')) | Some((_, '\n')) => { input_bytes.next(); },
                Some((i, 'd')) => {
                    let current_idx = *i;
                    self.push_keyword(Token::Define, KEY_DEF, &mut input_bytes, current_idx, input_string); 
                },
                Some((i, 'a')) => {
                    let current_idx = *i;
                    self.push_keyword(Token::As, KEY_AS, &mut input_bytes, current_idx, input_string); 
                },
                Some((i, 'e')) => {
                    let current_idx = *i;
                    self.push_keyword(Token::End, KEY_END, &mut input_bytes, current_idx, input_string); 
                },
                Some((_, '0'..='9')) => {
                    self.push_number(&mut input_bytes);
                },
                Some((_, 'a'..='z')) | 
                Some((_, 'A'..='Z')) | Some((_, '_'))=> {
                    self.push_identifier(&mut input_bytes);
                },
                
                _ => {unreachable!()}
            }
            
        } 
    }
}
