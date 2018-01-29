use std::str::FromStr;
use reader::regex::Captures;

extern crate regex;

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
            Some(self.tokens[self.position - 1].trim().to_string())
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

// Next time these will be objects not enums
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Symbol(String),
    Other(String),
    Odd(String),
    Keyword(String),
    UserKeyword(String),
    Number(i32),
    Closure(Vec<Token>, Vec<(Token, Token)>),
    List(Vec<Token>),
    Vector(Vec<Token>),
    HashMap(Vec<Token>),
    Error(String),
}

pub fn read_str(s: &str) -> Vec<Token> {
    let mut t = tokenizer(s);
    read_form(&mut t, "")
}

fn _read_form(r: &mut Reader, close_char: &str) -> Vec<Token> {
    r.next();
    let to_add = read_form(r, close_char);
    r.next();
    to_add
}

pub fn read_form(r: &mut Reader, close_char: &str) -> Vec<Token> {
    let mut result = vec![];
    while r.peek() != None && r.peek().unwrap() != close_char {
        result.push(if r.peek().unwrap() == "(" {
            Token::List(_read_form(r, ")"))
        } else if r.peek().unwrap() == "{" {
            Token::HashMap(_read_form(r, "}"))
        } else if r.peek().unwrap() == "[" {
            Token::Vector(_read_form(r, "]"))
        } else {
            read_atom(r)
        })
    }
    result
}

fn read_atom(reader: &mut Reader) -> Token {
    // just trim gives me a &str
    let s = &reader.next().unwrap().trim().to_string();

    if s == "def!" || s == "def" {
        return Token::Keyword("def".to_string());
    }
    if s == "let*" || s == "let" {
        return Token::Keyword("let".to_string());
    }
    if s == "list" {
        return Token::Keyword("list".to_string());
    }
    if s == "list?" {
        return Token::Keyword("list?".to_string());
    }
    if s == "empty?" {
        return Token::Keyword("empty?".to_string());
    }
    if s == "count" {
        return Token::Keyword("count".to_string());
    }
    if s == "false" {
        return Token::Keyword("false".to_string());
    }
    if s == "true" {
        return Token::Keyword("true".to_string());
    }
    if s == "nil" {
        return Token::Keyword("nil".to_string());
    }
    if s == "if" {
        return Token::Keyword("if".to_string());
    }
    if s == "fn" || s == "fn*" {
        return Token::Keyword("fn".to_string());
    }
    if s == "do" {
        return Token::Keyword("do".to_string());
    }
    if s == "prn" {
        return Token::Keyword("prn".to_string());
    }

    let odd_bit = regex!(r###"(~@)|['`~]"###);
    if let Some(odd) = odd_bit.find(s) {
        return Token::Odd(odd.as_str().to_string());
    }

    let keyword_regex = regex!(r":\w+[\w\d]*");
    let mat = keyword_regex.find(s);
    match mat {
        Some(m) => Token::UserKeyword(m.as_str().to_string()),
        _ => {
            let r = regex!(r"^\s*(?P<digits>-?\d+)$");
            let digits = r.find(s.as_ref());
            match digits {
                Some(x) => Token::Number(i32::from_str(x.as_str()).unwrap()),
                _ => match s.as_ref() {
                    "+" => Token::Symbol("+".to_string()),
                    "-" => Token::Symbol("-".to_string()),
                    "*" => Token::Symbol("*".to_string()),
                    "**" => Token::Symbol("**".to_string()),
                    "/" => Token::Symbol("/".to_string()),
                    "=" => Token::Keyword("=".to_string()),
                    ">" => Token::Keyword(">".to_string()),
                    "<" => Token::Keyword("<".to_string()),
                    ">=" => Token::Keyword(">=".to_string()),
                    "<=" => Token::Keyword("<=".to_string()),
                    s => Token::Other(s.to_string()),
                },
            }
        }
    }
}

fn update_state<'a>(s: &'a str, tokens: &mut Vec<String>, match_point: &Captures<'a>) -> &'a str {
    let new_s = &s[match_point.get(1).unwrap().end()..];
    tokens.push(match_point.get(1).unwrap().as_str().to_string());
    new_s
}

pub fn tokenizer(s_in: &str) -> Reader {
    let mut s = &s_in[0..];
    // Idea: consider merging this with main tokenization method?
    let brackets = regex!(r###"^[\s,]*([\(\)\{\}\[\]])[\s,]*"###);
    let digits = regex!(r"^[\s,]*(-?\d+)");
    let operands = regex!(r"^[\s,]*(\*{1,2}|[\+\-/])"); //{} is greedy to detect ** instead of: *
    let equalities = regex!(r"^[\s,]*((>=)|(<=)|[<>=])");
    let alphas = regex!(r###"^[\s,]*([?!\w\d:"\-\*]+)"###);
    let strings = regex!(r###"^[\s,]*("((\\")|[^"])*")"###);
    let odd_shit = regex!(r###"^[\s,]*((~@)|['`~])"###);
    let comment = regex!(r###"^[\s,]*(;.*)$"###);
    let mut tokens = vec![];
    let all_regexs = vec![
        &strings,
        &brackets,
        &odd_shit,
        &equalities,
        &digits,
        &operands,
        &alphas,
    ];

    let empty = regex!(r"^[\s,]+$");

    while !s.is_empty() && !empty.is_match(s) {
        if let Some(the_comment) = comment.captures(s) {
            s = &s[the_comment.get(1).unwrap().end()..];
        }
        for regex in &all_regexs {
            if let Some(rb) = regex.captures(s) {
                s = update_state(s, &mut tokens, &rb);
                break;
            }
        }
    }
    //println!("tokenizer: {:?}", tokens);
    Reader {
        tokens: tokens,
        position: 0,
    }
}

#[test]
fn test_tokenizer() {
    assert_eq!(tokenizer("34").tokens, vec!["34"]);
    assert_eq!(tokenizer("34,4").tokens, vec!["34", "4"]);
    assert_eq!(tokenizer("* 34 -4").tokens, vec!["*", "34", "-4"]);
    assert_eq!(tokenizer("*-34 4").tokens, vec!["*", "-34", "4"]);
    assert_eq!(tokenizer("/ 4 1").tokens, vec!["/", "4", "1"]);
    assert_eq!(tokenizer("** 1 2").tokens, vec!["**", "1", "2"]);
    assert_eq!(tokenizer(":kw").tokens, vec![":kw"]);
    assert_eq!(
        tokenizer("(1 2, 3,,,,),,").tokens,
        vec!["(", "1", "2", "3", ")"]
    );
    assert_eq!(tokenizer("abc").tokens, vec!["abc"]);
    assert_eq!(tokenizer("\"abc (hi)\"").tokens, vec!["\"abc (hi)\""]);
    assert_eq!(
        tokenizer("\"with \\\" a quote in\"").tokens,
        vec!["\"with \\\" a quote in\""]
    );
    assert_eq!(tokenizer("'1").tokens, vec!["'", "1"]);
    assert_eq!(tokenizer("`(1 2)").tokens, vec!["`", "(", "1", "2", ")"]);
    assert_eq!(tokenizer("1 ;some comment").tokens, vec!["1"]);
    assert_eq!(tokenizer("def! sd 3").tokens, vec!["def!", "sd", "3"]);
}

#[test]
fn test_read_atom() {
    fn _atom_helper(s: &str) -> Reader {
        Reader {
            tokens: vec![s.to_string()],
            position: 0,
        }
    }
    assert_eq!(
        read_atom(&mut _atom_helper(":hell33o ")),
        Token::UserKeyword(":hell33o".to_string())
    );
    assert_eq!(read_atom(&mut _atom_helper("-898")), Token::Number(-898));
    assert_eq!(
        read_atom(&mut _atom_helper("`")),
        Token::Odd("`".to_string())
    );
}
