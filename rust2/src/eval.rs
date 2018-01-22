use std::collections::HashMap;

use reader;

//pub type Callback = fn(reader::Token, &reader::Token) -> reader::Token;
/*
pub fn apply_sym_wrapper2(tokens : &mut Vec<reader::Token>, func_map: &mut HashMap<reader::Token, reader::Token>) -> reader::Token {
    let head = &tokens[0];
    return match head {
        &reader::Token::List(ref sublist) => apply_sym(None, None, reader::Token::List(vec![]), &mut (sublist.clone()), func_map),
        _ => apply_sym(None, None, reader::Token::List(vec![]), &mut tokens.clone(), func_map)
    }
}

pub fn apply_sym_wrapper(tokens : &mut Vec<reader::Token>, func_map: &mut HashMap<reader::Token, reader::Token>) -> reader::Token {
    if let reader::Token::List(ref sub_list) = tokens[0] {
        return apply_sym(None, None, reader::Token::Other("".to_string()), &mut sub_list.clone(), func_map)
    }
    return apply_sym(None, None, reader::Token::Other("".to_string()), tokens, func_map)
}*/

pub fn apply_sym_multi(
        group_type: reader::Token,
        tokens : &mut Vec<reader::Token>, 
        func_map: &mut HashMap<reader::Token, reader::Token>
) -> reader::Token {

    if tokens.len() == 0 {
        return reader::Token::List(vec![])
    }
    let mut head = tokens.remove(0);
    if let reader::Token::Keyword(keyword) = head {
        head = _process_keyword(keyword.as_ref(), tokens, func_map);
    };
    let head_applied = apply_sym_single(&head, func_map);

    match head_applied {
        reader::Token::Symbol(sym) => {
            let first = to_number(&mut tokens.remove(0), func_map);

            let n = match sym.as_ref() {
                "+" => tokens.iter().fold(first, |a,b| { a + to_number(b, func_map) } ),
                "-" => tokens.iter().fold(first, |a,b| { a - to_number(b, func_map) } ),
                "*" => tokens.iter().fold(first, |a,b| { a * to_number(b, func_map) } ),
                "**" => tokens.iter().fold(first, |a,b| { a.pow( to_number(b, func_map) as u32) }),
                "/" => tokens.iter().fold(first, |a,b| { a / to_number(b, func_map)}),
                _  => panic!("unknown operator {:?}", sym)
            };
            return reader::Token::Number(n)
        },
        
        _ => {
            // If there are no tokens left in this list we need to return the inside element
            // directly
            if tokens.len() == 0 {
                return head_applied
            }
            // greedily eat list until a list object.
            let ti = tokens.iter();
            let mut new_list = vec![head_applied];
            for nxt_token in ti {
                new_list.push(
                    match nxt_token {
                        &reader::Token::List(ref list) => {
                            apply_sym_multi(reader::Token::List(vec![]), &mut list.clone(), func_map)
                        },
                        &reader::Token::Vector(ref list) => {
                            apply_sym_multi(reader::Token::Vector(vec![]), &mut list.clone(), func_map)
                        },
                        &reader::Token::Other(_) => {
                            if func_map.contains_key(&nxt_token) {
                                apply_sym_single(&mut func_map[&nxt_token].clone(), func_map)
                            } else {
                                nxt_token.clone()
                            }
                        },
                        &_ => nxt_token.clone()
                    }
                );
            };
            match group_type {
                reader::Token::List(_) => return reader::Token::List(new_list),
                reader::Token::Vector(_) => return reader::Token::Vector(new_list),
                reader::Token::HashMap(_) => return reader::Token::HashMap(new_list),
                _ => panic!("Need to know which group type to use")
            }
        }
    }
}

pub fn apply_sym_single(
        head: &reader::Token, 
        func_map: &mut HashMap<reader::Token, reader::Token>
) -> reader::Token {
    println!("head: {:?}", head);

    match head {
        &reader::Token::Other(_) => {
            if func_map.contains_key(&head) {
                apply_sym_single(&mut func_map[&head].clone(), func_map)
            } else {
                head.clone()
            }
        },
        &reader::Token::List(ref list) => apply_sym_multi(reader::Token::List(vec![]), &mut list.clone(), func_map),
        &reader::Token::Vector(ref list) => apply_sym_multi(reader::Token::Vector(vec![]), &mut list.clone(), func_map),
        &reader::Token::HashMap(ref list) => apply_sym_multi(reader::Token::HashMap(vec![]), &mut list.clone(), func_map),
        &reader::Token::Symbol(_) => {
            /*if symbol.is_some() {
                panic!("Bad syntax used a {:?} and a {:?}", sym, symbol);
            }*/
            return head.clone()
        },
        _ => head.clone()
    }
}

pub fn to_number(token: &reader::Token, func_map: &mut HashMap<reader::Token, reader::Token>) -> i32 {
    let t2 = apply_sym_single(token, func_map);
    match t2  {
        reader::Token::Number(n) => n as i32,
        _ => panic!("Need a number type token: {:?}", token)
    }
}

fn _process_keyword(
        keyword : &str,
        tokens: &mut Vec<reader::Token>, 
        func_map: &mut HashMap<reader::Token, reader::Token>
) -> reader::Token {
    match keyword {
        "def" => {
            let var_name = tokens.remove(0);
            let var_value = tokens.remove(0);
            func_map.insert(var_name, var_value.clone());
            apply_sym_single(&var_value, func_map)
        },
        "let" => {
            let vars = match tokens.remove(0) {
                reader::Token::List(ref list) => list.clone(),
                reader::Token::Vector(ref list) => list.clone(),
                _ => panic!("Use of let requires a list")
            };
            let to_eval = tokens.remove(0);

            let mut count = 0;
            let mut new_func_map = func_map.clone();
            while count < vars.len() {
                let var_name = &vars[count];
                let var_value = &vars[count + 1];
                new_func_map.insert(var_name.clone(), var_value.clone());
                count += 2;
            }

            let res = apply_sym_single(&to_eval, &mut new_func_map);
            res
        },
        "list?" => {
            match tokens.remove(0) {
                reader::Token::List(_) => reader::Token::Keyword("true".to_string()),
                _ => reader::Token::Keyword("false".to_string())
            }
        },
        "empty?" => {
            match tokens.remove(0) {
                reader::Token::List(sublist) => {
                    match sublist.len() {
                        0 => reader::Token::Keyword("true".to_string()),
                        _ => reader::Token::Keyword("false".to_string())
                    }
                },
                reader::Token::Vector(sublist) => {
                    match sublist.len() {
                        0 => reader::Token::Keyword("true".to_string()),
                        _ => reader::Token::Keyword("false".to_string())
                    }
                },
                _ => panic!("Must call empty? on a list")
            }
        },
        "count" => {
            match tokens.remove(0) {
                reader::Token::List(sublist) => reader::Token::Number(sublist.len() as i32),
                reader::Token::Vector(sublist) => reader::Token::Number(sublist.len() as i32),
                _ => panic!("Must call count? on a list")
            }
        },
        "list" => {
            reader::Token::List(tokens.drain(..).collect())
        },
        "if"  => {
            let mut if_to_eval = tokens.remove(0);
            let mut if_true = tokens.remove(0);
            let mut if_false = tokens.remove(0);
            
            if _is_true(apply_sym_single(&mut if_to_eval, func_map)) {
                apply_sym_single(&mut if_true, func_map) 
            } else {
                apply_sym_single(&mut if_false, func_map)
            }
        },
        "false" => reader::Token::Keyword(keyword.to_string()),
        "true" => reader::Token::Keyword(keyword.to_string()),
        "nil" => reader::Token::Keyword(keyword.to_string()),
        "=" => _handle_comparison("=", tokens, func_map ),
        "<" => _handle_comparison("<", tokens, func_map ),
        ">" => _handle_comparison(">", tokens, func_map),
        "<=" => _handle_comparison("<=", tokens, func_map),
        ">=" => _handle_comparison(">=", tokens, func_map),
        _ => panic!("Unknown keyword {:?}", keyword)
    }
}

fn _handle_comparison(keyword :&str, tokens: &mut Vec<reader::Token>, func_map: &mut HashMap<reader::Token, reader::Token>) -> reader::Token {
    let mut first = tokens.remove(0);
    let mut second = tokens.remove(0);
    if _is_true_comparison(keyword, apply_sym_single(&mut first, func_map), apply_sym_single(&mut second, func_map)){
        reader::Token::Keyword("true".to_string())
    } else {
        reader::Token::Keyword("false".to_string())
    }
}

fn _is_true_comparison(comparison: &str, token_left: reader::Token, token_right: reader::Token) -> bool {
    if comparison == "=" && token_left == token_right {
        true
    } else if (comparison == "<=" || comparison == ">=") && token_left == token_right {
        true
    } else {
        let (new_token_left, new_token_right) = {
            if comparison == ">" || comparison == ">=" {
                (token_right, token_left)
            } else {
                (token_left, token_right)
            }
        };
        match (new_token_left, new_token_right) {
            (reader::Token::Number(n), reader::Token::Number(n2)) => n < n2,
            (a, b) => panic!("Uknown token generated for comparison {:?} {:?} {:?}", a, b, comparison)
        }
    }
}
            
fn _is_true(token: reader::Token) -> bool {
    println!("is_true: {:?}", token);
    match token {
        reader::Token::Keyword(s) => {
            match s.as_ref() {
                "false" => false,
                "nil" => false,
                _ => true
            }
        },
        reader::Token::Number(n) => n != 0,
        reader::Token::List(sublist) => sublist.len() != 0,
        _ => panic!("Uknown token generated in if {:?}", token)
    }
}
            
#[test]
fn test_handle_comparison() {
    assert!(_is_true_comparison("<", reader::Token::Number(5), reader::Token::Number(8)));
    assert!(!_is_true_comparison(">", reader::Token::Number(5), reader::Token::Number(8)));
    assert!(_is_true_comparison("<=", reader::Token::Number(5), reader::Token::Number(8)));
    assert!(!_is_true_comparison(">=", reader::Token::Number(5), reader::Token::Number(8)));
    assert!(_is_true_comparison(">=", reader::Token::Number(8), reader::Token::Number(8)));
    assert!(_is_true_comparison("<=", reader::Token::Number(8), reader::Token::Number(8)));
}

