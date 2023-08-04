use std::io::{self, Write};

use lexer::Lexer;
use parser::Parser;
use runtime::Env;

mod lexer;
mod parser;
mod runtime;

// fn main() {

//     let input_string = " a = 3.a";
//     let mut lexer = Lexer::new();
//     lexer.lex(input_string);

//     println!("{:?}", lexer.tokens);
//     println!("{:?}", lexer.errors);
// }

fn main() {

    let mut env = Env::new();

    loop {
        let mut input_string = String::new();
        
        env.print_prefix();

        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin().read_line(&mut input_string).expect("Failed to read input line");
        
        if input_string.eq("quit\n") {
            break;
        }

        if input_string.len() == 0 {
            continue;
        }

        let mut lexer = Lexer::new();
        lexer.lex(input_string.as_str());
        
        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);
    
        if lexer.errors.len() > 0 {
            for err in lexer.errors.iter() {
                println!("TOKENIZATION ERROR: {}", err);
            }
            continue;
        }

        // for token in lexer.tokens.iter() {
        //     println!("{:?}", token);
        // }
        
        if res.is_err() {
            println!("PARSING ERROR: {}", res.unwrap_err());
            continue;
        }

        // for stmt in parser.stmts.iter() {
        //     println!("{:?}", stmt);
        // }
        
        let res = env.interpret(parser.stmts);
        if res.is_err() {
            println!("RUNTIME ERROR: {}", res.unwrap_err());
            continue;
        }

    }

    


}
