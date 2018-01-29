use std::collections::HashMap;

use reader;
use printer;

//pub type Callback = fn(reader::Token, &reader::Token) -> reader::Token;

pub fn apply_sym_multi(
    group_type: &reader::Token,
    tokens: &mut Vec<reader::Token>,
    func_map: &mut HashMap<reader::Token, reader::Token>,
) -> reader::Token {
    if tokens.is_empty() {
        return reader::Token::List(vec![]);
    }
    let old_head = _remove_or_nil(tokens);
    let head = {
        if let reader::Token::Keyword(ref keyword) = old_head {
            _process_keyword(keyword.as_ref(), tokens, func_map)
        } else {
            old_head
        }
    };
    let head_applied = apply_sym_single(&head, func_map);

    match head_applied {
        reader::Token::Closure(contents, env_as_tuple_list) => {
            let params = &contents[0];
            let body = &contents[1];
            let mut new_env: HashMap<reader::Token, reader::Token> =
                env_as_tuple_list.into_iter().collect();

            let mut param_iter = {
                if let reader::Token::Vector(ref param_list) = *params {
                    param_list.into_iter().peekable()
                } else if let reader::Token::List(ref param_list) = *params {
                    param_list.into_iter().peekable()
                } else {
                    panic!(
                        "first token after a func def must be a vector got: {:?}",
                        params
                    )
                }
            };

            while !tokens.is_empty() && param_iter.peek().is_some() {
                // Eagerly eval the contents of a function to allow recursion to work
                // We need a rethink on eager vs lazy here.
                let eval_now = apply_sym_single(&_remove_or_nil(tokens), func_map);
                let pn = param_iter.next().unwrap();
                new_env.insert(pn.clone(), eval_now.clone());
            }

            if tokens.is_empty() && param_iter.peek().is_none() {
                apply_sym_single(body, &mut new_env)
            } else {
                //contents must be rebuilt here to hold the new environment
                // we really shouldn't have so many clones we could do this smarter
                let contents2 = vec![
                    reader::Token::List(param_iter.cloned().collect()),
                    body.clone(),
                ];
                reader::Token::Closure(
                    contents2,
                    new_env
                        .iter()
                        .map(|(a, b)| (a.clone(), b.clone()))
                        .collect(),
                )
            }
        }

        reader::Token::Symbol(ref sym) => apply_symbol(sym, tokens, func_map),

        _ => {
            // If there are no tokens left in this list we need to return the inside element
            // directly
            if tokens.is_empty() {
                return head_applied;
            }
            // greedily eat list until a list object.
            let ti = tokens.iter();
            let mut new_list = vec![head_applied];
            for nxt_token in ti {
                new_list.push(match *nxt_token {
                    reader::Token::List(ref list) => {
                        apply_sym_multi(&reader::Token::List(vec![]), &mut list.clone(), func_map)
                    }
                    reader::Token::Vector(ref list) => {
                        apply_sym_multi(&reader::Token::Vector(vec![]), &mut list.clone(), func_map)
                    }
                    reader::Token::Other(_) => {
                        if func_map.contains_key(nxt_token) {
                            apply_sym_single(&func_map[nxt_token].clone(), func_map)
                        } else {
                            nxt_token.clone()
                        }
                    }
                    _ => nxt_token.clone(),
                });
            }
            match *group_type {
                reader::Token::List(_) => reader::Token::List(new_list),
                reader::Token::Vector(_) => reader::Token::Vector(new_list),
                reader::Token::HashMap(_) => reader::Token::HashMap(new_list),
                _ => panic!("Need to know which group type to use"),
            }
        }
    }
}

pub fn apply_sym_single(
    head: &reader::Token,
    func_map: &mut HashMap<reader::Token, reader::Token>,
) -> reader::Token {
    match *head {
        reader::Token::Other(ref var_name) => {
            if func_map.contains_key(head) {
                let tmp = &mut func_map[head].clone();
                apply_sym_single(tmp, func_map)
            } else {
                reader::Token::Error(format!("Unknown variable/definition: {}", var_name))
            }
        }
        reader::Token::List(ref list) => {
            apply_sym_multi(&reader::Token::List(vec![]), &mut list.clone(), func_map)
        }
        reader::Token::Vector(ref list) => {
            apply_sym_multi(&reader::Token::Vector(vec![]), &mut list.clone(), func_map)
        }
        reader::Token::HashMap(ref list) => {
            apply_sym_multi(&reader::Token::HashMap(vec![]), &mut list.clone(), func_map)
        }
        _ => head.clone(),
    }
}

fn really_do_sym(sym: &str, sm: i32, n: i32) -> reader::Token {
    match sym {
        "+" => reader::Token::Number(sm + n),
        "-" => reader::Token::Number(sm - n),
        "*" => reader::Token::Number(sm * n),
        "**" => reader::Token::Number(sm.pow(n as u32)),
        "/" => reader::Token::Number(sm / n),
        _ => reader::Token::Error(format!("Unknown symbol: {}", sym)),
    }
}

pub fn do_sym(sym: &str, a: Option<reader::Token>, b: reader::Token) -> reader::Token {
    match a {
        None => b,
        Some(some_a) => match (some_a, b) {
            (reader::Token::Error(e), _) => reader::Token::Error(e),
            (_, reader::Token::Error(e)) => reader::Token::Error(e),
            (reader::Token::Number(sum), reader::Token::Number(n)) => really_do_sym(sym, sum, n),
            (reader::Token::Vector(sum), reader::Token::Vector(vec_parts)) => {
                reader::Token::Vector(
                    vec_parts
                        .into_iter()
                        .zip(sum)
                        .map(|(v, s)| do_sym(sym, Some(s), v))
                        .collect(),
                )
            }
            (sum, reader::Token::Vector(vec_parts)) => reader::Token::Vector(
                vec_parts
                    .into_iter()
                    .map(|v| do_sym(sym, Some(sum.clone()), v))
                    .collect(),
            ),
            (reader::Token::Vector(sum), any) => reader::Token::Vector(
                sum.into_iter()
                    .map(|a| do_sym(sym, Some(a), any.clone()))
                    .collect(),
            ),
            (any_a, any_b) => reader::Token::Error(format!(
                "Could not match types ({:?} {} {:?})",
                any_a.clone(),
                sym,
                any_b.clone()
            )),
        },
    }
}

pub fn apply_symbol(
    sym: &str,
    tokens: &mut Vec<reader::Token>,
    func_map: &mut HashMap<reader::Token, reader::Token>,
) -> reader::Token {
    let mut sum = None;
    for t in tokens {
        let nxt = apply_sym_single(t, func_map);
        match nxt {
            reader::Token::Error(e) => return reader::Token::Error(e),
            _ => {
                sum = Some(do_sym(sym, sum, nxt));
            }
        }
    }
    sum.unwrap()
}

fn _process_keyword(
    keyword: &str,
    tokens: &mut Vec<reader::Token>,
    func_map: &mut HashMap<reader::Token, reader::Token>,
) -> reader::Token {
    match keyword {
        "def" => {
            let var_name = _remove_or_nil(tokens);
            let var_value = _remove_or_nil(tokens);
            func_map.insert(var_name, var_value.clone());
            apply_sym_single(&var_value, func_map)
        }
        "let" => {
            let vars = match _remove_or_nil(tokens) {
                reader::Token::List(ref list) => list.clone(),
                reader::Token::Vector(ref list) => list.clone(),
                _ => panic!("Use of let requires a list"),
            };
            let to_eval = _remove_or_nil(tokens);

            let mut count = 0;
            let mut new_func_map = func_map.clone();
            while count < vars.len() {
                let var_name = &vars[count];
                let var_value = &vars[count + 1];
                new_func_map.insert(var_name.clone(), var_value.clone());
                count += 2;
            }

            apply_sym_single(&to_eval, &mut new_func_map)
        }
        "fn" => {
            let params = _remove_or_nil(tokens);
            let func_body = _remove_or_nil(tokens);
            let lst = func_map
                .iter()
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect();
            reader::Token::Closure(vec![params, func_body], lst)
        }
        "do" => apply_sym_single(&_remove_or_nil(tokens), func_map),
        "list?" => match _remove_or_nil(tokens) {
            reader::Token::List(_) => reader::Token::Keyword("true".to_string()),
            _ => reader::Token::Keyword("false".to_string()),
        },
        "empty?" => match _remove_or_nil(tokens) {
            reader::Token::List(sublist) | reader::Token::Vector(sublist) => match sublist.len() {
                0 => reader::Token::Keyword("true".to_string()),
                _ => reader::Token::Keyword("false".to_string()),
            },
            _ => panic!("Must call empty? on a list"),
        },
        "count" => match _remove_or_nil(tokens) {
            reader::Token::List(sublist) | reader::Token::Vector(sublist) => {
                reader::Token::Number(sublist.len() as i32)
            }
            _ => reader::Token::Number(0),
        },
        "list" => reader::Token::List(tokens.drain(..).collect()),
        "if" => {
            let mut if_to_eval = _remove_or_nil(tokens);
            let mut if_true = _remove_or_nil(tokens);
            let mut if_false = _remove_or_nil(tokens);

            if _is_true(apply_sym_single(&if_to_eval, func_map)) {
                apply_sym_single(&if_true, func_map)
            } else {
                apply_sym_single(&if_false, func_map)
            }
        }
        "prn" => {
            let to_print: Vec<reader::Token> = tokens.drain(..).collect();
            println!("{:?}", printer::pr_str(&to_print));
            reader::Token::Keyword("nil".to_string())
        }
        "false" => reader::Token::Keyword(keyword.to_string()),
        "true" => reader::Token::Keyword(keyword.to_string()),
        "nil" => reader::Token::Keyword(keyword.to_string()),
        "=" => _handle_comparison("=", tokens, func_map),
        "<" => _handle_comparison("<", tokens, func_map),
        ">" => _handle_comparison(">", tokens, func_map),
        "<=" => _handle_comparison("<=", tokens, func_map),
        ">=" => _handle_comparison(">=", tokens, func_map),
        _ => panic!("Unknown keyword {:?}", keyword),
    }
}

fn _handle_comparison(
    keyword: &str,
    tokens: &mut Vec<reader::Token>,
    func_map: &mut HashMap<reader::Token, reader::Token>,
) -> reader::Token {
    let first = _remove_or_nil(tokens);
    let second = _remove_or_nil(tokens);
    if _is_true_comparison(
        keyword,
        apply_sym_single(&first, func_map),
        apply_sym_single(&second, func_map),
    ) {
        reader::Token::Keyword("true".to_string())
    } else {
        reader::Token::Keyword("false".to_string())
    }
}

fn _is_true_comparison(
    comparison: &str,
    token_left: reader::Token,
    token_right: reader::Token,
) -> bool {
    if comparison == "=" {
        token_left == token_right
    } else if (comparison == "<=" || comparison == ">=") && token_left == token_right {
        true
    } else {
        // This let flips the tokens on a '>' so we can always compare as if a '<'
        let (new_token_left, new_token_right) = {
            if comparison == ">" || comparison == ">=" {
                (token_right, token_left)
            } else {
                (token_left, token_right)
            }
        };
        match (new_token_left, new_token_right) {
            (reader::Token::Number(n), reader::Token::Number(n2)) => n < n2,
            (a, b) => panic!(
                "Uknown token generated for comparison {:?} {:?} {:?}",
                a, b, comparison
            ),
        }
    }
}

fn _is_true(token: reader::Token) -> bool {
    match token {
        reader::Token::Other(s) => match s.as_ref() {
            "" => false,
            "\"\"" => false,
            _ => true,
        },
        reader::Token::Keyword(s) => match s.as_ref() {
            "false" => false,
            "nil" => false,
            _ => true,
        },
        reader::Token::Number(n) => n != 0,
        reader::Token::List(sublist) => !sublist.is_empty(),
        _ => panic!("Uknown token passed to if {:?}", token),
    }
}

pub fn _remove_or_nil(tokens: &mut Vec<reader::Token>) -> reader::Token {
    if !tokens.is_empty() {
        tokens.remove(0)
    } else {
        reader::Token::Keyword("nil".to_string())
    }
}

#[test]
fn test_handle_comparison() {
    assert!(_is_true_comparison(
        "<",
        reader::Token::Number(5),
        reader::Token::Number(8)
    ));
    assert!(!_is_true_comparison(
        ">",
        reader::Token::Number(5),
        reader::Token::Number(8)
    ));
    assert!(_is_true_comparison(
        "<=",
        reader::Token::Number(5),
        reader::Token::Number(8)
    ));
    assert!(!_is_true_comparison(
        ">=",
        reader::Token::Number(5),
        reader::Token::Number(8)
    ));
    assert!(_is_true_comparison(
        ">=",
        reader::Token::Number(8),
        reader::Token::Number(8)
    ));
    assert!(_is_true_comparison(
        "<=",
        reader::Token::Number(8),
        reader::Token::Number(8)
    ));
    assert!(_is_true_comparison(
        "=",
        reader::Token::Number(8),
        reader::Token::Number(8)
    ));
    assert!(!_is_true_comparison(
        "=",
        reader::Token::Number(4),
        reader::Token::Number(8)
    ));
}

#[test]
fn test_do_sym() {
    assert_eq!(
        do_sym(
            "+",
            Some(reader::Token::Number(5)),
            reader::Token::Number(6)
        ),
        reader::Token::Number(11)
    );
}

#[test]
fn test_do_sym_vec_first() {
    assert_eq!(
        do_sym(
            "*",
            Some(reader::Token::Vector(vec![
                reader::Token::Number(5),
                reader::Token::Number(6),
            ])),
            reader::Token::Number(2)
        ),
        reader::Token::Vector(vec![reader::Token::Number(10), reader::Token::Number(12)])
    );
}

#[test]
fn test_do_sym_vec_last() {
    assert_eq!(
        do_sym(
            "-",
            Some(reader::Token::Number(3)),
            reader::Token::Vector(vec![reader::Token::Number(5), reader::Token::Number(6)])
        ),
        reader::Token::Vector(vec![reader::Token::Number(-2), reader::Token::Number(-3)])
    );
}

#[test]
fn test_do_sym_vec() {
    assert_eq!(
        do_sym(
            "/",
            Some(reader::Token::Vector(vec![
                reader::Token::Number(50),
                reader::Token::Number(60),
            ])),
            reader::Token::Vector(vec![reader::Token::Number(2), reader::Token::Number(6)])
        ),
        reader::Token::Vector(vec![reader::Token::Number(25), reader::Token::Number(10)])
    );
}

#[test]
fn test_sum_bit() {
    let mut enviro: HashMap<reader::Token, reader::Token> = HashMap::new();
    assert_eq!(
        apply_symbol(
            "+",
            &mut vec![reader::Token::Number(5), reader::Token::Number(6)],
            &mut enviro
        ),
        reader::Token::Number(11)
    );
}
