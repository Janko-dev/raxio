use std::io::{self, Write};
use std::{env, fs};

use lexer::Lexer;
use parser::Parser;
use runtime::Env;

mod lexer;
mod parser;
mod runtime;
mod error;

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
            println!("{}", err);
        }
    }

    let mut parser = Parser::new();
    let res = parser.parse(&mut lexer);

    if res.is_err() {
        println!("{}", res.unwrap_err());
    }

    let res = env.interpret(parser.stmts);
    
    if res.is_err() {
        println!("{}", res.unwrap_err());
    }
}

fn start_repl() {
    let mut env = Env::new();
    println!("Welcome to the REPL environment of raxio.");
    println!("Enter \"quit\" to stop the REPL environment.");
    println!("Enter \"help\" to see an overview of raxio syntax.");
    println!("Enter \"undo\" during mattern patching to undo the current expression.");

    loop {
        let mut input_string = String::new();
        
        env.print_prefix();

        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin().read_line(&mut input_string).expect("Failed to read input line");
        
        match &input_string.as_str() {
            &"quit\r\n" | 
            &"quit\n" | 
            &"quit" => { break; },
            &"help\r\n" | 
            &"help\n" | 
            &"help" => { print_help(); continue; },
            &"undo\r\n" | 
            &"undo\n" | 
            &"undo" => { 
                env.pop_expr();
                continue;
            },
            _ => {}
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
                println!("{}", err);
            }
            continue;
        }
        
        if res.is_err() {
            println!("{}", res.unwrap_err());
            continue;
        }

        let res = env.interpret(parser.stmts);
        if res.is_err() {
            println!("{}", res.unwrap_err());
            env.is_matching = false;
            continue;
        }

    }
}

fn print_help() {
    println!("Raxio syntax:");
    println!("To define a rule, use");
    println!("    - def [YOUR_RULE_NAME] as [LEFT_EXPR] => [RIGHT_EXPR]");
    println!("      YOUR_RULE_NAME is an alphanumeric identifier.");
    println!("      LEFT_EXPR is the expression to match against.");
    println!("      RIGHT_EXPR is the expression to produce if left expression was matched.\n");
    println!("To start pattern matching an expression, use either");
    println!("    - a variable, e.g., x, foo, abc, etc.; or");
    println!("    - a functor, e.g., f(x), g(h(x, y)), foo(bar(baz)), etc.; or");
    println!("    - binary mathematical in-fix operations, '+', '-', '*', '/'.");
    println!("      e.g., a + b (which gets translated to the functor add(a, b))\n");
    println!("To apply a rule during pattern matching of expession, enter either");
    println!("    - predefined identifier of a rule followed a number indicating at which depth to apply the rule");
    println!("      e.g., [YOUR_RULE_NAME] at [DEPTH]; or");
    println!("    - an in-line rule without an identifier followed by a number indicating at which depth to apply the rule");
    println!("      e.g., [LEFT_EXPR] => [RIGHT_EXPR] at [DEPTH]\n");
}
