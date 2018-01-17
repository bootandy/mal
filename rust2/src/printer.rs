
use reader;

pub fn pr_str(tokens: &Vec<reader::Token>) -> String {
    let mut result = String::from("");
    let mut ugly_counter = 0;

    for r in tokens.iter() {
        //println!("{:?}", r);
        result += match r {
            &reader::Token::Number(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Symbol(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Keyword(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Other(ref n) => " ".to_string() + n.to_string().as_ref(),
            &reader::Token::Odd(ref odd) => {
                ugly_counter = 2;
                match odd.as_ref() {
                    "'" => " (quote",
                    "`" => " (quasiquote",
                    "~@" => " (splice-unquote",
                    "~" => " (unquote",
                    _ => panic!("unknown token {:?}", odd)
                }.to_string()
            },
            &reader::Token::List(ref list) => " (".to_string() + pr_str(list).as_ref() + ")",
            &reader::Token::Vector(ref list) => " [".to_string() + pr_str(list).as_ref() + "]",
            &reader::Token::HashMap(ref list) => " {".to_string() + pr_str(list).as_ref() + "}",
            &reader::Token::Crap() => "".to_string()
        }.as_ref();
        ugly_counter -= 1;
        if ugly_counter == 0 {
            result = result.trim().to_string() + ")";
        }
    }
    result.trim().to_string()
}

