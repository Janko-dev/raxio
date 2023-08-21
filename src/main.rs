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

    
    for err in lexer.errors.iter() {
        println!("{}", err);
    }

    let mut parser = Parser::new();
    let res = parser.parse(&mut lexer);

    if let Err(e) = res {
        println!("{}", e);
    }

    let res = env.interpret(parser.stmts);
    
    if env.warnings.len() > 0 {
        for warn in env.warnings.iter() {
            println!("{}", warn);
        }
        env.warnings.clear();
    }

    if let Err(e) = res {
        println!("{}", e);
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
        
        let input_string = input_string.trim();
        
        if input_string.len() == 0 {
            continue;
        }
        
        match input_string {
            "quit" => { return; }, 
            "help" => { print_help(); continue; },
            "undo" => { env.pop_expr(); continue; },
            _ => {}
        }

        let mut lexer = Lexer::new();
        lexer.lex(input_string);
        
        let mut parser = Parser::new();
        let res = parser.parse(&mut lexer);
    
        if lexer.errors.len() > 0 {
            for err in lexer.errors.iter() {
                println!("{}", err);
            }
            continue;
        }
        
        if let Err(e) = res {
            println!("{}", e);
            continue;
        }

        let res = env.interpret(parser.stmts);

        if env.warnings.len() > 0 {
            for warn in env.warnings.iter() {
                println!("{}", warn);
            }
            env.warnings.clear();
        }

        if let Err(e) = res {
            println!("{}", e);
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
    println!("    - binary arithmetic in-fix operations, '+', '-', '*', '/'.");
    println!("      e.g., a + b (which gets translated to the functor add(a, b))\n");
    println!("To apply a rule during pattern matching of expession, enter either");
    println!("    - predefined identifier of a rule followed by a number indicating at which depth to apply the rule");
    println!("      e.g., apply [YOUR_RULE_NAME] at [DEPTH]; or");
    println!("    - an in-line rule without an identifier followed by a number indicating at which depth to apply the rule");
    println!("      e.g., [LEFT_EXPR] => [RIGHT_EXPR] at [DEPTH]\n");
}
