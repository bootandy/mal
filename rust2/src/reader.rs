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
    Odd(String),
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
    let odd_bit = regex!(r###"(~@)|['`~]"###);
    if let Some(odd) = odd_bit.find(s) {
        return Token::Odd(odd.as_str().to_string());
    }

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

fn update_state<'a>(s :&'a str, tokens :&mut Vec<String>, match_point : &Captures<'a>) -> &'a str {
    let new_s = &s[match_point.get(1).unwrap().end()..];
    tokens.push(match_point.get(1).unwrap().as_str().to_string());
    new_s
}

pub fn tokenizer(s_in: &str) -> Reader {
    let mut s = &s_in[0..];
    let brackets = regex!(r###"^[\s,]*([\(\)\{\}\[\]])[\s,]*"###);
    let digits = regex!(r"^[\s,]*(-?\d+)");
    let operands = regex!(r"^[\s,]*(\*{1,2}|[\+\-\\])"); //{} is greedy to detect ** instead of: *
    let alphas = regex!(r###"^[\s,]*([\w\d:"-]+)"###);
    let strings = regex!(r###"^[\s,]*("((\\")|[^"])*")"###);
    let odd_shit = regex!(r###"^[\s,]*((~@)|['`~])"###);
    let comment = regex!(r###"^[\s,]*(;.*)$"###);
    let mut tokens = vec![];
    let all_regexs = vec![&strings, &brackets, &odd_shit, &digits, &operands, &alphas] ;

    let empty = regex!(r"^[\s,]+$");

    while s.len() > 0 && !empty.is_match(s) {

        if let Some(the_comment) = comment.captures(s) {
            s = &s[the_comment.get(1).unwrap().end()..];
        } 
        for regex in &all_regexs {
            if let Some(rb) = regex.captures(s) {
                s = update_state(s, &mut tokens, &rb);
                break
            }
        }
    } 
    //println!("{:?}", tokens);
    Reader{tokens: tokens, position: 0}
}


#[test]
fn test_tokenizer() {
    assert_eq!(tokenizer("34").tokens, vec!["34"]);
    assert_eq!(tokenizer("34,4").tokens, vec!["34", "4"]);
    assert_eq!(tokenizer("* 34 -4").tokens, vec!["*", "34", "-4"]);
    assert_eq!(tokenizer("*-34 4").tokens, vec!["*", "-34", "4"]);
    assert_eq!(tokenizer("** 1 2").tokens, vec!["**", "1", "2"]);
    assert_eq!(tokenizer(":kw").tokens, vec![":kw"]);
    assert_eq!(tokenizer("(1 2, 3,,,,),,").tokens, vec!["(", "1", "2", "3", ")"]);
    assert_eq!(tokenizer("abc").tokens, vec!["abc"]);
    assert_eq!(tokenizer("\"abc (hi)\"").tokens, vec!["\"abc (hi)\""]);
    assert_eq!(tokenizer("\"with \\\" a quote in\"").tokens, vec!["\"with \\\" a quote in\""]);
    assert_eq!(tokenizer("'1").tokens, vec!["'", "1"]);
    assert_eq!(tokenizer("`(1 2)").tokens, vec!["`", "(", "1", "2", ")"]);
    assert_eq!(tokenizer("1 ;some comment").tokens, vec!["1"]);
}

#[test]
fn test_read_atom() {
    assert_eq!(_read_atom(":hell33o "),  Token::Keyword(":hell33o".to_string()));
    assert_eq!(_read_atom("-898"),  Token::Number(-898));
    assert_eq!(_read_atom("`"),  Token::Odd("`".to_string()));
}

