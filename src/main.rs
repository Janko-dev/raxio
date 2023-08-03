use lexer::Lexer;
use parser::Parser;


mod lexer;
mod parser;

fn main() {

    let input_string = "def switch as f(x, y) => f(y, x)";
    let mut lexer = Lexer::new();
    lexer.lex(input_string);
    
    // for token in lexer.tokens.iter() {
    //     println!("{:?}", token);
    // }

    let mut parser = Parser::new();
    let res = parser.parse(&mut lexer);

    if res.is_err() || lexer.errors.len() > 0 {
        for err in lexer.errors.iter() {
            println!("TOKENIZATION ERROR: {}", err);
        }
        println!("PARSING ERROR: {}", res.unwrap_err());
        
    } else {
        for stmt in parser.stmts.iter() {
            println!("{:?}", stmt);
        }
    }

}
