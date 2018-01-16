
use reader;

pub fn pr_str(tokens: &Vec<reader::Token>) -> String {
    let mut result = String::from("");
    for r in tokens.iter() {
        result += match r {
            &reader::Token::Number(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Symbol(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Keyword(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Other(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::List(ref list) => " (".to_string() + pr_str(list).as_ref() + ")",
            &reader::Token::Vector(ref list) => " [".to_string() + pr_str(list).as_ref() + "]",
            &reader::Token::HashMap(ref list) => " {".to_string() + pr_str(list).as_ref() + "}",
            &reader::Token::Crap() => "".to_string()
        }.as_ref();
    }
    result.trim().to_string()
}

