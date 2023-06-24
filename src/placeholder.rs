pub use logos::*;
use ini::Ini;

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, Logos)]
pub enum Token {
    #[regex(r"!\{[\S]+\}", |lex| let a = lex.slice(); a[2..a.len()-1].to_string())]
    Placeholder(String),

    #[regex(r"[\s\S]+", |lex| lex.slice().to_string())]
    Text(String)
}

pub fn placeholder_process(orig: &str, ini: Ini) -> Option<String> {
    let mut lex = Token::lexer(orig);
    let mut buf = String::new();
    while let Some(t) = lex.next() {
        match t.unwrap() {
            Token::Text(t) => buf.push_str(&t),
            Token::Placeholder(k) => {
                let ks = k.split(".").collect::<Vec<&str>>();
                let s = ini.section(if !k.contains(".") { None } else { Some(ks[..ks.len()-1].join(".")) });
                let s = if s;
            }
        }
    }

    Some(buf)
}
