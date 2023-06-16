use super::lexer::*;

pub fn to_html(src: &str) -> String {
    let lex = Token::lexer(src);
    let mut buf = Buffer::new(lex);
    toks_to_html(&mut buf)
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

fn toks_to_html(lex: &mut Buffer) -> String {
    let mut buf = format!(
r#"<!DOCTYPE html><html><head>
<link rel="stylesheet" href="test.css">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/styles/atom-one-dark.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/highlight.min.js"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.7/dist/katex.min.css">
<script>hljs.highlightAll();</script></head><body>"#).replace("\n", "");
    let mut scope = vec![0_usize; std::mem::variant_count::<Token>()];

    let mut indentation = 0;
    let mut dot_lists = Vec::new();
    let mut in_list = false;

    while let Some(i) = lex.next() {
        match i {
            Token::Text(s)      => buf.push_str(&s),
            Token::Escape(c)    => match c {
                '<' => buf.push_str("&lt;"),
                '>' => buf.push_str("&gt;"),
                _ => buf.push(c)
            },

            Token::NewLine(ind) => {
                indentation = ind;

                let mut br = true;

                // headings
                let heading_s = &mut scope[Token::Heading(0).as_usize()];
                if *heading_s != 0 {
                    buf.push_str(&format!("</h{}>", heading_s));
                    *heading_s = 0;
                    br = false;
                }

                // code blocks
                if scope[Token::CodeBlock("".to_string()).as_usize()] != 0 {
                    buf.push_str(&format!("\n{}", " ".repeat(ind)));
                    br = false;
                }

                // dot lists
                let dl_s = &mut scope[Token::DotList.as_usize()];

                if in_list && lex.peek().unwrap_or(Token::Bang) != Token::DotList {
                    *dl_s = 2;
                    in_list = false;
                }

                if *dl_s == 1 {
                    buf.push_str("</li>");
                    br = false;
                } else if *dl_s == 2 {
                    buf.push_str("</li></ul>");
                    br = false;
                }
                *dl_s = 0;

                // blockquotes
                let bq_s = &mut scope[Token::BlockQuote.as_usize()];
                if *bq_s == 1 && lex.peek().unwrap_or(Token::Bang) != Token::BlockQuote {
                    buf.push_str("</div>");
                    *bq_s = 0;
                    br = false;
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

            Token::CodeBlock(ref l) => {
                let f = &mut scope[i.as_usize()];
                if *f == 0 {
                    buf.push_str(&format!("<pre><code class=\"language-{l}\">"))
                } else {
                    buf.push_str("</code></pre>")
                }
                *f ^= 1;
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
                if lex.peek() == Some(Token::BlockQuote) {
                    buf.push('>');
                    lex.ind += 1;
                } else {
                    scope[i.as_usize()] = 1
                }
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

            Token::DotList => {
                let c = *dot_lists.last().unwrap_or(&0);
                if indentation > c {
                    buf.push_str("<ul>");
                    dot_lists.push(indentation);
                    in_list = true;
                }
                buf.push_str("<li>");
                scope[i.as_usize()] = 1;
                if indentation < c {
                    scope[i.as_usize()] = 2;
                    dot_lists.pop();
                }
            },
            Token::BlockQuote => {
                buf.push_str(r#"<div class="blockquote">"#);
                scope[i.as_usize()] = 1;
            },
            Token::KaTeX(latex) => {
                let opts = katex::Opts::builder().output_type(katex::opts::OutputType::Html).build().unwrap();
                buf.push_str(&katex::render_with_opts(&latex, opts).unwrap())
            },
        }
    }

    buf.push_str("</body></html>");

    buf
}
