use std::str::FromStr;

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
    read_form(&mut t)
}

pub fn read_form(r: &mut Reader) -> Vec<Token> {
    let mut result = vec![];
    while r.peek() != None && r.peek().unwrap() != ")" {
        result.push( 
            if r.peek().unwrap() == "(" {
                r.next();
                let to_add = Token::List(read_form(r));
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

pub fn tokenizer(s: &str) -> Reader {
    let r = regex!(r###"[\s,]*(~@|[\[\]{}()"`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"###);
    let kill_comma = regex!(r",");
    let mut result = vec![];
    for i in r.find_iter(s) {
        let val = i.as_str();
        let no_comma = kill_comma.replace_all(val, "");
        result.push(String::from(no_comma.trim()));
    }

    // The above super regex doesn't split out operators from operands
    // We do that here:
    let is_op = regex!(r"^[\+\*/\-]");
    let is_negative_number = regex!(r"^[\-]\s*\d+");
    let mut result2 = vec![];

    for r in result.iter() {
        let mut r2 :String = r.chars().skip(0).collect();
        while is_op.is_match(&r2) && !is_negative_number.is_match(&r2) {
            let first_char : String = r2.chars().take(1).collect();
            result2.push(String::from(first_char));
            r2 = r2.chars().skip(1).collect();
        }

        if r2 != "" {
            result2.push(r2);
        }
    }
    Reader{tokens: result2, position: 0}
}


#[test]
fn test_tokenizer() {
    assert_eq!(tokenizer("34").tokens, vec!["34"]);
    assert_eq!(tokenizer("34,4").tokens, vec!["34", "4"]);
    assert_eq!(tokenizer("* 34 -4").tokens, vec!["*", "34", "-4"]);
    assert_eq!(tokenizer("*-34 4").tokens, vec!["*", "-34", "4"]);
    assert_eq!(tokenizer("** 1 2").tokens, vec!["*", "*", "1", "2"]);
    assert_eq!(tokenizer("(1 2, 3,,,,),,").tokens, vec!["(", "1", "2", "3", ")"]);
}

#[test]
fn test_read_atom() {
    assert_eq!(_read_atom(":hell33o "),  Token::Keyword(":hell33o".to_string()));
    assert_eq!(_read_atom("-898"),  Token::Number(-898));
}

