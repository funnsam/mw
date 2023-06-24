pub use logos::*;
use ini::Ini;

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, Logos)]
pub enum PlaceholderToken {
    #[regex(r"\{[^\}]+\}", priority = 999, callback = |lex| let a = lex.slice(); a[1..a.len()-1].to_string())]
    Placeholder(String),

    #[regex(r"[^\{]+", |lex| lex.slice().to_string())]
    Text(String)
}

pub fn placeholder_process(tem: &str, ini: &Ini, gini: &Ini, body: &str) -> Result<String, PlaceholderError> {
    let mut lex = PlaceholderToken::lexer(tem);
    let mut buf = String::new();
    while let Some(t) = lex.next() {
        match t.unwrap() {
            PlaceholderToken::Text(t) => buf.push_str(&t),
            PlaceholderToken::Placeholder(k) => {
                if &k == "body" {
                    buf.push_str(body);
                    continue;
                }

                let ks = k.split(".").collect::<Vec<&str>>();
                let sn = if !k.contains(".") { None } else { Some(ks[..ks.len()-1].join(".")) };
                let sl =  ini.section(sn.clone());
                let sg = gini.section(sn.clone());
                let kn = ks.last().unwrap();
                let cl = if sl.is_some() { sl.unwrap().get(kn) } else { None };
                let cg = if sg.is_some() { sg.unwrap().get(kn) } else { None };
                if cl.is_none() && cg.is_none() {
                    return Err(PlaceholderError::CannotFindKey(sn.unwrap_or("root".to_string()), kn.to_string()))
                } else {
                    buf.push_str(cl.unwrap_or(cg.unwrap()))
                }
            }
        }
    }

    Ok(buf)
}

#[derive(Clone, Debug)]
pub enum PlaceholderError {
    CannotFindSection(String),
    CannotFindKey(String, String),
}
