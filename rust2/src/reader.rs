use std::str::FromStr;
use reader::regex::Captures;

extern crate regex;

//use regex;

macro_rules! regex {
     ($e:expr) => (regex::Regex::new($e).unwrap())
}
    
#[derive(Debug, Clone)]
pub struct Reader {
    tokens: Vec<String>,
    position: usize,
}

impl Reader {
    fn next(&mut self) -> Option<String> {
        if self.position < self.tokens.len() {
            self.position += 1;
            Some(self.tokens[self.position-1].trim().to_string())
        } else {
            None
        }
    }
    fn peek(&self) -> Option<String> {
        if self.position < self.tokens.len() {
            Some(self.tokens[self.position].trim().to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Symbol(String),
    Crap(),
    Other(String),
    Keyword(String),
    Number(i32),
    List(Vec<Token>),
    Vector(Vec<Token>),
    HashMap(Vec<Token>)
}

pub fn read_str(s :&str) -> Vec<Token> {
    let mut t = tokenizer(s);
    read_form(&mut t, "")
}

pub fn read_form(r: &mut Reader, close_char: &str) -> Vec<Token> {
    let mut result = vec![];
    while r.peek() != None && r.peek().unwrap() != close_char {
        result.push( 
            if r.peek().unwrap() == "(" {
                r.next();
                let to_add = Token::List(read_form(r, ")"));
                r.next();
                to_add
            } else if r.peek().unwrap() == "{" {
                r.next();
                let to_add = Token::HashMap(read_form(r, "}"));
                r.next();
                to_add
            } else if r.peek().unwrap() == "[" {
                r.next();
                let to_add = Token::Vector(read_form(r, "]"));
                r.next();
                to_add
            } else {
                read_atom(r)
            }
        )
    }
    //println!("{:?}", result);
    result
}

fn read_atom(reader : &mut Reader) -> Token {
    let s = reader.next().unwrap();
    _read_atom(&s)
}

fn _read_atom(s: &str) -> Token {
    let keyword_regex = regex!(r":\w+[\w\d]*");
    let mat = keyword_regex.find(s);
    match mat {
        Some(m) => return Token::Keyword(m.as_str().to_string()),
        _ => {
            let r = regex!(r"^\s*(?P<digits>-?\d+)$");
            let cap = r.find(s.as_ref());
            match cap {
                Some(x) => return Token::Number( i32::from_str(x.as_str()).unwrap() ),
                _ =>  {
                    match s.as_ref() {
                        "+" => Token::Symbol("+".to_string()),
                        "-" => Token::Symbol("-".to_string()),
                        "*" => Token::Symbol("*".to_string()),
                        "**" => Token::Symbol("**".to_string()),
                        "/" => Token::Symbol("/".to_string()),
                        s => Token::Other(s.to_string())
                    }
                }
            }
        }
    }
}

fn update_state<'a>(s :&'a str, match_point : &Captures<'a>) -> (&'a str, &'a str) {
    let new_s = &s[match_point.get(1).unwrap().end()..];
    let to_add = match_point.get(1).unwrap().as_str();
    (new_s, to_add) 
}

pub fn tokenizer(s_in: &str) -> Reader {
    let mut s = &s_in[0..];
    let brackets = regex!(r###"^[\s,]*([\(\)\{\}\[\]])[\s,]*"###);
    let digits = regex!(r"^[\s,]*(-?\d+)");
    let operands = regex!(r"^[\s,]*(\*{1,2}|[\+\-\\])"); //{} is greedy to detect ** instead of: *
    let alphas = regex!(r###"^[\s,]*([\w\d:"-]+)"###);
    let mut tokens = vec![];

    while s.len() > 0 {
        let rb = brackets.captures(s);

        tokens.push( String::from(
            match rb {
                Some(bracket) => {
                    let (ss, res) = update_state(s, &bracket);
                    s = ss;
                    res
                },
                _ => {
                    let a_number = digits.captures(s);
                    match a_number {
                        Some(n) => {
                            let (ss, res) = update_state(s, &n);
                            s = ss;
                            res
                        },
                        _ => {
                            let a_symbol = operands.captures(s);
                            match a_symbol {
                                Some(sym) => {
                                    let (ss, res) = update_state(s, &sym);
                                    s = ss;
                                    res
                                },
                                _ => {
                                    let a_other = alphas.captures(s);
                                    match a_other {
                                        Some(alp) => {
                                            let (ss, res) = update_state(s, &alp);
                                            s = ss;
                                            res
                                        },
                                        _ => {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ));
    }

    Reader{tokens: tokens, position: 0}
}


#[test]
fn test_tokenizer() {
    assert_eq!(tokenizer("34").tokens, vec!["34"]);
    assert_eq!(tokenizer("34,4").tokens, vec!["34", "4"]);
    assert_eq!(tokenizer("* 34 -4").tokens, vec!["*", "34", "-4"]);
    assert_eq!(tokenizer("*-34 4").tokens, vec!["*", "-34", "4"]);
    assert_eq!(tokenizer("** 1 2").tokens, vec!["**", "1", "2"]);
    assert_eq!(tokenizer("(1 2, 3,,,,),,").tokens, vec!["(", "1", "2", "3", ")"]);
    assert_eq!(tokenizer("abc").tokens, vec!["abc"]);
    assert_eq!(tokenizer("\"abc (hi)\"").tokens, vec!["\"abc (hi)\""]);
}

#[test]
fn test_read_atom() {
    assert_eq!(_read_atom(":hell33o "),  Token::Keyword(":hell33o".to_string()));
    assert_eq!(_read_atom("-898"),  Token::Number(-898));
}

