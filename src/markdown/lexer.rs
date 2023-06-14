pub use logos::*;

#[repr(usize)]
#[derive(Debug, Clone, Logos)]
pub enum Token {
    #[regex(r"\\.", |lex| lex.slice().chars().nth(1))]
    Escape(char),

    #[token("*")]
    Italic,
    #[token("**")]
    Bold,

    #[regex(r"```[^\n]*[\n]?", |lex| lex.slice()[3..].trim().to_string())]
    CodeBlock(String),
    #[token("`")]
    InlineCode,

    #[token("!")]
    Bang,
    #[regex(r"\[[^\n]*\]", |lex| let s = lex.slice(); s[1..s.len()-1].to_string())]
    LinkName(String),
    #[regex(r"\([^\n]*\)", |lex| let s = lex.slice(); s[1..s.len()-1].to_string())]
    LinkURL(String),

    #[regex(r"[#]+", |lex| lex.slice().len())]
    Heading(usize),

    #[regex(r"[^\n*\\#`\[\]\(\)]*", |lex| lex.slice().to_string())]
    Text(String),

    #[token("\n")]
    NewLine,
}

impl Token {
    pub fn as_usize(&self) -> usize {
        unsafe { *<*const _>::from(self).cast::<usize>() }
    }
}
