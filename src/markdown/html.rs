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
        for i in l { ts.push(i.unwrap()); }
        Self { ts, ind: 0 }
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
    let mut buf = format!(r#"<!DOCTYPE html><html><head><link rel="stylesheet" href="test.css"></head><body>"#);
    let mut scope = vec![0_usize; std::mem::variant_count::<Token>()];

    for i in &mut *lex {
        eprintln!("{i:?}");
        match i {
            Token::Text(s)      => buf.push_str(&s),
            Token::Escape(c)    => buf.push(c),

            Token::NewLine      => {
                let cb_s = scope[Token::CodeBlock("".to_string()).as_usize()];
                let heading_s = &mut scope[Token::Heading(0).as_usize()];
                if *heading_s != 0 {
                    buf.push_str(&format!("</h{}>", heading_s));
                    *heading_s = 0;
                } else if cb_s != 0 {
                    buf.push('\n')
                } else {
                    buf.push_str("<br>")
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

            Token::Bang => scope[i.as_usize()] = 1,
            Token::LinkName(name) => {
                let url = match lex.next().unwrap()
                    { Token::LinkURL(u) => u, _ => panic!("Unexpected token after URL name") };

                if scope[Token::Bang.as_usize()] == 1 {
                    buf.push_str(&format!(r#"<img src="{url}" alt="{name}">"#));
                    scope[Token::Bang.as_usize()] = 0;
                } else {
                    buf.push_str(&format!(r#"<a src="{url}">{name}</a>"#))
                }
            },

            Token::LinkURL(_) => panic!("Unexpected round brackets"),

            Token::Heading(h) => {
                buf.push_str(&format!("<h{h}>"));
                scope[i.as_usize()] = h;
            },
        }
    }

    buf.push_str("</body></html>");

    buf
}
