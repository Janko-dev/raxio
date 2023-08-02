use lexer::TokenList;

mod lexer;

fn main() {

    let input_string = "hello def 3.14    as => , , ,,".to_string();
    let mut tokenlist = TokenList::new();
    tokenlist.lex(input_string.as_str());

    for token in tokenlist.tokens.iter(){
        println!("{:?}", token);
    }
}
