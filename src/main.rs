use std::io::{self, Write};
use std::{env, fs};

use lexer::Lexer;
use parser::Parser;
use runtime::Env;

mod lexer;
mod parser;
mod runtime;

// fn main() {

//     let input_string = "
//             f(A)
//             f(x) => g(x, x)
//     ";
//     let mut lexer = Lexer::new();
//     lexer.lex(input_string);
    
//     let mut parser = Parser::new();
//     let res = parser.parse(&mut lexer);

//     if res.is_err() {
//         println!("PARSING ERROR: {}", res.unwrap_err());
//     }

//     let mut env = Env::new();
//     let res = env.interpret(parser.stmts);
//     println!("{:?}", env.current_expr);
//     if res.is_err() {
//         println!("RUNTIME ERROR: {}", res.unwrap_err());
//     }
// }

fn main() {

    let mut args: Vec<String> = env::args().collect();

    match args.len() {
        1 => { start_repl(); },
        2 => { interpret_file(args.swap_remove(1)); },
        _ => { usage(); }
    }
}

fn usage() {
    println!("Usage:");
    println!("    Provide file for interpretation");
    println!("        $ ./raxio [FILE_NAME]");
    println!("    When no arguments are provided, enter REPL mode");
    println!("        $ ./raxio");
}

fn interpret_file(file_name: String) {
    
    let input_string = match fs::read_to_string(file_name) {
        Ok(s) => s,
        Err(msg) => panic!("{}", msg)
    };

    let mut env = Env::new();

    let mut lexer = Lexer::new();
    lexer.lex(&input_string);

    if lexer.errors.len() > 0 {
        for err in lexer.errors.iter() {
            println!("TOKENIZATION ERROR: {}", err);
        }
    }

    let mut parser = Parser::new();
    let res = parser.parse(&mut lexer);

    if res.is_err() {
        println!("PARSING ERROR: {}", res.unwrap_err());
    }

    let res = env.interpret(parser.stmts);
    
    if res.is_err() {
        println!("RUNTIME ERROR: {}", res.unwrap_err());
    }
}

fn start_repl() {
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
            env.is_matching = false;
            continue;
        }

    }
}
