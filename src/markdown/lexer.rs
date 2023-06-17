pub use logos::*;

#[repr(usize)]
#[derive(Debug, Clone, PartialEq, Eq, Logos)]
pub enum Token {
    #[regex(r"\\[\s\S]", priority = 9999, callback = |lex| lex.slice().chars().nth(1))]
    Escape(char),

    #[token("*")]
    Italic,
    #[token("**")]
    Bold,

    // TODO: single and double back ticks
    #[regex(r"```[^\n]*\n([^`])*```", |lex| let (a, b) = lex.slice()[3..].split_once("\n").unwrap(); (a.to_string(), b[..b.len()-3].to_string()))]
    CodeBlock((String, String)),
    #[token("`")]
    InlineCode,

    #[token("-")]
    DotList,

    #[token(">")]
    BlockQuote,

    #[token("!")]
    Bang,
    #[regex(r"\[[^\n\]]+\]", |lex| let s = lex.slice(); s[1..s.len()-1].to_string())]
    LinkName(String),
    #[regex(r"\([^\n\)]+\)", |lex| let s = lex.slice(); s[1..s.len()-1].to_string())]
    LinkURL(String),

    #[regex(r"[#]+", |lex| lex.slice().len())]
    Heading(usize),

    #[regex(r"\$(\\\$|[^\$])*\$", callback = |lex| let s = lex.slice().to_string(); s[1..s.len()-1].to_string())]
    MathExpr(String),

    #[regex(r"!\{(\\\}|[^\}])+\}", priority = 99, callback = |lex| let s = lex.slice().to_string(); s[2..s.len()-1].replace("\\}", "}").to_string())]
    TextBlock(String),

    #[regex(r"[^\n*\\#`\[\]\(\)!>\-\$]*", priority = 0, callback = |lex| lex.slice().to_string())]
    Text(String),

    #[regex(r"\n[\t ]*", |lex| lex.slice().replace("\t", "    ")[1..].len())]
    NewLine(usize),
}

impl Token {
    pub fn as_usize(&self) -> usize {
        unsafe { *<*const _>::from(self).cast::<usize>() }
    }
}
