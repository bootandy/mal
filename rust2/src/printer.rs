use reader;

pub fn pr_str(tokens: &[reader::Token]) -> String {
    let mut result = String::from("");
    let mut ugly_counter = 0;

    for r in tokens.iter() {
        result += format!(
            " {}",
            match *r {
                reader::Token::Number(ref n) => n.to_string(),
                reader::Token::Symbol(ref n) => n.to_string(),
                reader::Token::UserKeyword(ref n) => n.to_string(),
                reader::Token::Keyword(ref n) => n.to_string(),
                reader::Token::Closure(_, _) => "#<function>".to_string(),
                reader::Token::Other(ref n) => n.to_string(),
                reader::Token::Odd(ref odd) => {
                    ugly_counter = 2;
                    match odd.as_ref() {
                        "'" => "(quote",
                        "`" => "(quasiquote",
                        "~@" => "(splice-unquote",
                        "~" => "(unquote",
                        _ => panic!("unknown token {:?}", odd),
                    }.to_string()
                }
                reader::Token::List(ref list) => format!("({})", pr_str(list)),
                reader::Token::Vector(ref list) => format!("[{}]", pr_str(list)),
                reader::Token::HashMap(ref list) => format!("{{{}}}", pr_str(list)),
                reader::Token::Error(ref e) => format!("Error: {:?} ", e),
            }
        ).as_ref();
        ugly_counter -= 1;
        if ugly_counter == 0 {
            result = result.trim().to_string() + ")";
        }
    }
    result.trim().to_string()
}
