
use std::io;
use std::io::Write;
use std::collections::HashMap;

mod reader;
mod printer;
mod eval;

fn get_input() -> String  {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
            Ok(_n) => {
            }
            Err(error) => {
                println!("error: {}", error);
                input = "Figure out Option / Errors later".to_string();
            }
    }
    input
}

fn eval(tokens: Vec<reader::Token>, enviro : &mut HashMap<reader::Token, reader::Token>) -> Vec<reader::Token> {
    vec![eval::apply_sym_wrapper(&tokens, enviro)]
}

fn print(data: Vec<reader::Token>) -> String {
    return printer::pr_str(&data);
}

fn read(s: &str) -> Vec<reader::Token> {
    return reader::read_str(s);
}

fn rep(s: &str, enviro : &mut HashMap<reader::Token, reader::Token>) -> String {
    let ss = read(s);
    let s_eval = eval(ss, enviro);
    print(s_eval)
}

pub fn main() {
    let mut enviro  = HashMap::new();
    //let &mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    loop {
        print!("user> ");
        io::stdout().flush().unwrap();
        let buffer = get_input();
        let buffer_trim = buffer.trim();
        println!("{}", rep(buffer_trim, &mut enviro))
    }
}

#[test]
fn test_ting() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(*-3 6)", &mut enviro), "-18");
}

#[test]
fn test_rm_comma() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(1 2, 3,,,,),,", &mut enviro), "(1 2 3)");
}
#[test]
fn test_kw() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(:kw1 :kw2)", &mut enviro), "(:kw1 :kw2)");
}

#[test]
fn test_parans() {
    //change: we added parans in later version. 
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("abc (with parans)", &mut enviro), "(abc (with parans))"); 
}

#[test]
fn test_no_whitespace() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("\"abc\"", &mut enviro), "\"abc\"");
}

#[test]
fn test_double_star() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("** 2 3", &mut enviro), "8");
}

#[test]
fn test_basic_def() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("def! a 3", &mut enviro), "3");
    assert_eq!(rep("a", &mut enviro), "3");
}

#[test]
fn test_complex_def() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("def! a (+4 5 (*2 3))", &mut enviro), "15");
    assert_eq!(rep("a", &mut enviro), "15");
}

