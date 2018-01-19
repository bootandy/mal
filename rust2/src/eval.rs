use std::collections::HashMap;

use reader;

//pub type Callback = fn(reader::Token, &reader::Token) -> reader::Token;

pub fn apply_sym_wrapper2(tokens : &[reader::Token], func_map: &mut HashMap<reader::Token, reader::Token>) -> reader::Token {
    let hack = match tokens[0] {
        reader::Token::List(ref sub_list) => apply_sym(None, None, reader::Token::List(vec![]), &sub_list, func_map),
        _ => apply_sym(None, None, reader::Token::List(vec![]), &tokens, func_map),
    };
    match hack {
        reader::Token::List(ref sub_list) => {
            if sub_list.len() == 1 {
                sub_list[0].clone()
            } else {
                hack.clone()
            }
        },
        _ => hack.clone()
    }
            
}

pub fn apply_sym_wrapper(tokens : &[reader::Token], func_map: &mut HashMap<reader::Token, reader::Token>) -> reader::Token {
    match tokens[0] {
        reader::Token::List(ref sub_list) => apply_sym(None, None, reader::Token::Other("".to_string()), &sub_list, func_map),
        _ => apply_sym(None, None, reader::Token::Other("".to_string()), &tokens, func_map),
    }
}

pub fn apply_sym(
        symbol : Option<&str>, 
        acc : Option<i32>, 
        group_type: reader::Token,
        token_param : &[reader::Token], 
        func_map: &mut HashMap<reader::Token, reader::Token>
) -> reader::Token {
    let mut tokens = token_param;
    //println!("apply_sym {:?}", token_param);

    if tokens.len() == 0 {
        match acc {
            Some(ac) => return reader::Token::Number(ac),
            None => return reader::Token::List(vec![])
        }
    }

    let to_add = match &tokens[0] {
        &reader::Token::Keyword(ref keyword) => {
            match keyword.as_ref() {
                "def" => {
                    let var_name = tokens[1].clone();
                    let var_value = tokens[2].clone();
                    tokens = &tokens[2..];   // we will drop the first token in the vec when we recur
                    func_map.insert(var_name, var_value.clone());
                    apply_sym_wrapper(&vec![var_value], func_map)
                },
                "let" => {
                    let vars = match &tokens[1] {
                        &reader::Token::List(ref list) => list.clone(),
                        &reader::Token::Vector(ref list) => list.clone(),
                        _ => panic!("Use of let requires a list")
                    };
                    let to_eval = &tokens[2];

                    let mut count = 0;
                    let mut new_func_map = func_map.clone();
                    while count < vars.len() {
                        let var_name = &vars[count];
                        let var_value = &vars[count+1];
                        new_func_map.insert(var_name.clone(), var_value.clone());
                        count += 2;
                    }
                    tokens = &tokens[2..]; // we will drop the first token in the vec when we recur

                    let res = apply_sym_wrapper(&vec![to_eval.clone()], &mut new_func_map);
                    res
                },
                _ => panic!("Unknown keyword {:?}", keyword)
            }
        },
        &reader::Token::Other(_) => {
            if func_map.contains_key(&tokens[0]) {
                apply_sym_wrapper(&vec![func_map[&tokens[0]].clone()], func_map)
            } else {
                tokens[0].clone()
            }
        },
        &reader::Token::List(ref list) => apply_sym(None, None, reader::Token::List(vec![]), &list, func_map),
        &reader::Token::Vector(ref list) => apply_sym(None, None, reader::Token::Vector(vec![]), &list, func_map),
        &reader::Token::HashMap(ref list) => apply_sym(None, None, reader::Token::HashMap(vec![]), &list, func_map),
        &reader::Token::Symbol(ref sym) => {
            /*if symbol.is_some() {
                panic!("Bad syntax used a {:?} and a {:?}", sym, symbol);
            }*/
            return apply_sym(Some(sym), acc, group_type, &tokens[1..], func_map);
        },
        _ => tokens[0].clone()
    };

    match symbol {
        Some(sym) => {
            // If we are type number AND we have a symbol:
            let new_acc = match (acc, to_add) {
                // If we have an accumulator
                (Some(n_acc), reader::Token::Number(n_new))  => {
                    match sym {
                        "+" => n_acc + n_new,
                        "-" => n_acc - n_new,
                        "*" => n_acc * n_new,
                        "**" => n_acc.pow(n_new as u32),
                        "/" => n_acc / n_new,
                        _  => panic!("unknown operator {:?}", sym)
                    }
                },
                // No accumulator -> Lets populate it with the first value.
                (None, reader::Token::Number(n2)) => n2,
                (Some(_), x)  => panic!("cant add {:?}", x),
                (_, _) => panic!("no")
            };
            return apply_sym(symbol, Some(new_acc), group_type, &tokens[1..], func_map)
        },
        
        None => {
            match group_type {
                reader::Token::Other(_) => return to_add,
                _ => {
                    // greedily eat list until a list object.
                    let ti = tokens[1..].iter();
                    let mut new_list = vec![to_add];
                    for nxt_token in ti {
                        new_list.push(
                            match nxt_token {
                                &reader::Token::List(ref list) => {
                                    apply_sym(None, None, reader::Token::List(vec![]),  &list, func_map)
                                },
                                &reader::Token::Vector(ref list) => {
                                    apply_sym(None, None, reader::Token::Vector(vec![]), &list, func_map)
                                },
                                &reader::Token::Other(_) => {
                                    if func_map.contains_key(&nxt_token) {
                                        apply_sym_wrapper(&vec![func_map[&nxt_token].clone()], func_map)
                                    } else {
                                        tokens[0].clone()
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
    }
    
}

