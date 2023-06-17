use super::lexer::*;
use mathjax::MathJax;

pub fn to_html(src: &str, preamble: &str) -> String {
    let lex = Token::lexer(src);
    let mut buf = Buffer::new(lex);
    toks_to_html(&mut buf, preamble)
}

struct Buffer {
    ts: Vec<Token>,
    ind: usize
}

impl Buffer {
    fn new(l: Lexer<Token>) -> Self {
        let mut ts = Vec::new();
        for i in l {
            eprintln!("{i:?}");
            ts.push(i.unwrap());
        }
        Self { ts, ind: 0 }
    }
    fn peek(&self) -> Option<Token> {
        self.ts.get(self.ind).cloned()
    }
}

impl Iterator for Buffer {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let a = self.ts.get(self.ind);
        self.ind += 1;
        a.cloned()
    }
}

lazy_static::lazy_static! {
    static ref MATHJAX: MathJax = MathJax::new().unwrap();
}

fn toks_to_html(lex: &mut Buffer, preamble: &str) -> String {
    let mut buf = format!(
r#"<!DOCTYPE html><html><head>
<link rel="stylesheet" href="test.css">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/styles/atom-one-dark.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/highlight.min.js"></script>
<script>hljs.highlightAll();</script></head><body>"#).replace("\n", "");
    let mut scope = vec![0_usize; std::mem::variant_count::<Token>()];

//    let mut indentation = 0;
    let mut dot_lists = Vec::new();

    while let Some(i) = lex.next() {
        match i {
            Token::Text(ref s)  => buf.push_str(&sanitize(s)),
            Token::TextBlock(s) => buf.push_str(&s),
            Token::Escape(c)    => buf.push(c),

            Token::NewLine(ind) => {
                // indentation = ind;

                let mut br = true;

                // headings
                let heading_s = &mut scope[Token::Heading(0).as_usize()];
                if *heading_s != 0 {
                    buf.push_str(&format!("</h{}>", heading_s));
                    *heading_s = 0;
                    br = false;
                }

                // dot lists
                let c = *dot_lists.last().unwrap_or(&0);
                let dl_s = &mut scope[Token::DotList.as_usize()];

                if ind < c {
                    while ind < *dot_lists.last().unwrap_or(&0) {
                        buf.push_str("</li></ul>");
                        dot_lists.pop().unwrap_or(0);
                    }
                    br = false;
                } else if *dl_s == 1 {
                    buf.push_str("</li>");
                    br = false;
                }
                *dl_s = 0;

                // blockquotes
                let bq_s = &mut scope[Token::BlockQuote.as_usize()];

                if lex.peek().unwrap_or(Token::Bang) == Token::DotList {
                    if ind > c {
                        buf.push_str("<ul>");
                        dot_lists.push(ind);
                    }
                    buf.push_str("<li>");
                    *bq_s = 1;
                    lex.ind += 1;
                }

                if *bq_s == 1 && lex.peek().unwrap_or(Token::Bang) != Token::BlockQuote {
                    buf.push_str("</div>");
                    *bq_s = 0;
                    br = false;
                } else if *bq_s == 0 && lex.peek().unwrap_or(Token::Bang) == Token::BlockQuote {
                    buf.push_str(&format!(r#"<div class="blockquote">"#));
                    lex.ind += 1;
                    *bq_s = 1;
                    br = false;
                } else if lex.peek().unwrap_or(Token::Bang) == Token::BlockQuote {
                    lex.ind += 1;
                }
                
                // normal
                if br {
                    buf.push_str(&format!("<br>{}", "&nbsp;".repeat(ind)));
                }
            },

            Token::Bold => {
                let f = &mut scope[i.as_usize()];
                if *f == 0 {
                    buf.push_str("<b>")
                } else {
                    buf.push_str("</b>")
                }
                *f ^= 1;
            },
            Token::Italic => {
                let f = &mut scope[i.as_usize()];
                if *f == 0 {
                    buf.push_str("<i>")
                } else {
                    buf.push_str("</i>")
                }
                *f ^= 1;
            },
            Token::Strike => {
                let f = &mut scope[i.as_usize()];
                if *f == 0 {
                    buf.push_str("<del>")
                } else {
                    buf.push_str("</del>")
                }
                *f ^= 1;
            },

            Token::CodeBlock((ref l, ref c)) => {
                buf.push_str(&format!("<pre><code class=\"language-{l}\">{}</code></pre>", sanitize(c)));
            },

            Token::InlineCode => {
                let f = &mut scope[i.as_usize()];
                if *f == 0 {
                    buf.push_str("<code>")
                } else {
                    buf.push_str("</code>")
                }
                *f ^= 1;
            },

            Token::Bang => {
                scope[i.as_usize()] = 1
            },
            Token::LinkName(name) => {
                let url = match lex.next().unwrap()
                    { Token::LinkURL(u) => u, _ => panic!("Unexpected token after URL name") };

                if scope[Token::Bang.as_usize()] == 1 {
                    buf.push_str(&format!(r#"<img src="{url}" alt="{name}">"#));
                    scope[Token::Bang.as_usize()] = 0;
                } else {
                    buf.push_str(&format!(r#"<a href="{url}">{name}</a>"#))
                }
            },

            Token::LinkURL(_) => panic!("Unexpected round brackets"),

            Token::Heading(h) => {
                buf.push_str(&format!("<h{h}>"));
                scope[i.as_usize()] = h;
            },

            Token::DotList      => buf.push_str("-"),
            Token::BlockQuote   => buf.push_str("&gt;"),
            Token::MathExpr(expr) => {
                buf.push_str(
                    &MATHJAX.render(&format!("{preamble}\n{expr}")).unwrap().into_raw()
                );
            },
        }
    }

    buf.push_str("</body></html>");

    buf
}

pub fn sanitize(s: &str) -> String {
    s.replace("&", "&amp;")
     .replace("<", "&lt;")
     .replace(">", "&gt;")
     .replace(" ", "&nbsp;")
}
