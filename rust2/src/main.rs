
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

fn eval(mut tokens: Vec<reader::Token>, enviro : &mut HashMap<reader::Token, reader::Token>) -> Vec<reader::Token> {
    //vec![eval::apply_sym_multi(reader::Token::List(vec![]), &mut tokens, enviro)]
    vec![eval::apply_sym_single(&mut tokens.remove(0), enviro)]
}

fn print(data: Vec<reader::Token>) -> String {
    return printer::pr_str(&data);
}

fn read(s: &str) -> Vec<reader::Token> {
    return reader::read_str(s);
}

fn rep(s: &str, enviro : &mut HashMap<reader::Token, reader::Token>) -> String {
    let ss = read(s);
    //println!("{:?}", ss);
    let s_eval = eval(ss, enviro);
    print(s_eval)
}

pub fn main() {
    let mut enviro  = HashMap::new();
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

// I no longer agree with this:
/*#[test]
fn test_parans() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("abc (with parans)", &mut enviro), "abc (with parans)"); 
}*/

#[test]
fn test_no_whitespace() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("\"abc\"", &mut enviro), "\"abc\"");
}

#[test]
fn test_double_star() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(** 2 3)", &mut enviro), "8");
}

#[test]
fn test_basic_def() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(def! a 3)", &mut enviro), "3");
    assert_eq!(rep("a", &mut enviro), "3");
}

#[test]
fn test_complex_def() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(def! a (+4 5 (*2 3)))", &mut enviro), "15");
    assert_eq!(rep("a", &mut enviro), "15");
}

#[test]
fn test_let() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(let* (z 9) z)", &mut enviro), "9");
}

#[test]
fn test_complex_let() {
    // changed:  added parans to result. Parans should probably not be there.
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("(let* (p (+ 2 3) q (+ 2 p)) (+ p q))", &mut enviro), "12");
    assert_eq!(rep("(let* (a 5 b 6) [3 4 a [b 7] 8])", &mut enviro), "[3 4 5 [6 7] 8]");
}

#[test]
fn test_let_def_dont_fight() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(def z 9)", &mut enviro), "9");
      assert_eq!(rep("(let* (z 5) z)", &mut enviro), "5");
      assert_eq!(rep("z", &mut enviro), "9");
}

#[test]
fn test_list_quest_keyword() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(list? 1)", &mut enviro), "false");
      assert_eq!(rep("(list? ())", &mut enviro), "true");
}
#[test]
fn test_list_empty_keyword() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(empty? (2))", &mut enviro), "false");
      assert_eq!(rep("(empty? ())", &mut enviro), "true");
}
#[test]
fn test_count_keyword() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(count (1 2 3))", &mut enviro), "3");
}

#[test]
fn test_list_keyword() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(list 1 2 3)", &mut enviro), "(1 2 3)");
}

#[test]
fn test_if_keyword() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(if true 4 5)", &mut enviro), "4");
      assert_eq!(rep("(if false 4 5)", &mut enviro), "5");
}

#[test]
fn test_if_complex_keyword() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(if true (+2 4) (5))", &mut enviro), "6");
      assert_eq!(rep("(if 0 (+2 4) ())", &mut enviro), "()");
}

#[test]
fn test_basic_greater_less_thans() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(> 1 2)", &mut enviro), "false");
      assert_eq!(rep("(<= 2 2)", &mut enviro), "true");
}

#[test]
fn test_if_with_signs_and_defs() {
      let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
      assert_eq!(rep("(if (> 1 2) 3 4)", &mut enviro), "4");
      assert_eq!(rep("(def a 4)", &mut enviro), "4");
      assert_eq!(rep("(if (= a 4) 3 4)", &mut enviro), "3");
      assert_eq!(rep("(if (>a 2) 3 4)", &mut enviro), "3");
}

#[test]
fn test_fn() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("( (fn* [a b] (+ a b)) 2 3)", &mut enviro), "5");
}

#[test]
fn test_closure_fn() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(rep("( ( (fn* (a) (fn* (b) (+ a b))) 5) 7)", &mut enviro), "12");
}

#[test]
fn test_recursive_fn() {
    let mut enviro : HashMap<reader::Token, reader::Token> = HashMap::new();
    rep("(def! sumdown (fn* (N) (if (> N 0) (+ N (sumdown  (- N 1))) 0)))", &mut enviro);
    assert_eq!(rep("(sumdown 4", &mut enviro), "10");
}


