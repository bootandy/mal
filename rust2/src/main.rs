
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

fn eval(tokens: Vec<reader::Token>, enviro : &mut HashMap<reader::Token, eval::Callback>) -> Vec<reader::Token> {
    vec![eval::apply_sym_wrapper(&tokens, enviro)]
}

fn print(data: Vec<reader::Token>) -> String {
    return printer::pr_str(&data);
}

fn read(s: &str) -> Vec<reader::Token> {
    return reader::read_str(s);
}

fn rep(s: &str) -> String {
    let mut enviro : HashMap<reader::Token, eval::Callback> = HashMap::new();
    //eval::setup_base_functions(&mut enviro);
    
    let ss = read(s);
    let s_eval = eval(ss, &mut enviro);
    print(s_eval)
}

pub fn main() {
    loop {
        print!("user> ");
        io::stdout().flush().unwrap();
        let buffer = get_input();
        let buffer_trim = buffer.trim();
        println!("{}", rep(buffer_trim))
    }
}

#[test]
fn test_ting() {
    assert_eq!(rep("(*-3 6)"), "(* -3 6)")
}

#[test]
fn test_rm_comma() {
    assert_eq!(rep("(1 2, 3,,,,),,"), "(1 2 3)")
}
#[test]
fn test_kw() {
    assert_eq!(rep("(:kw1 :kw2)"), "(:kw1 :kw2)");
}

#[test]
fn test_parans() {
    assert_eq!(rep("abc (with parans)"), "abc (with parans)");
}

#[test]
fn test_no_whitespace() {
    assert_eq!(rep("\"abc\""), "\"abc\"");
}

#[test]
fn test_double_star() {
    assert_eq!(rep("** 1 2"), "** 1 2");
}

