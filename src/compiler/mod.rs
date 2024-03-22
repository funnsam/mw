use std::{path::*, fs::*};
use markdown::{mdast::*, *};

mod page_opts;

pub struct CompileOptions {
    pub md_options: ParseOptions,
}

pub fn compile(path: PathBuf, content: &str, options: &CompileOptions) {
    let ast = to_mdast(content, &options.md_options).unwrap();

    println!("{ast:?}");

    let mut html = String::new();
    let popt = to_html(&ast, &mut html);

    println!("{} {:?}", html, popt);
}

pub fn to_html(ast: &Node, acc: &mut String) -> Option<page_opts::PageOptions> {
    macro_rules! start_ended_parent {
        ($start: tt $children: tt $end: tt) => {{
            *acc += &format!($start);

            for c in $children.iter() {
                to_html(c, acc);
            }

            *acc += &format!($end);

            None
        }};
    }
    macro_rules! start_ended_value {
        ($start: tt $value: tt $end: tt) => {{
            *acc += &format!($start);
            *acc += &html_escape::encode_text($value);
            *acc += &format!($end);

            None
        }};
    }

    match ast {
        Node::Root(Root { children, .. }) => {
            let mut opt = page_opts::PageOptions::default();

            for c in children.iter() {
                match c {
                    Node::Toml(Toml { value, .. }) => {
                        opt = toml::from_str(&value).unwrap();
                        continue;
                    },
                    _ => { to_html(c, acc); },
                }
            }

            Some(opt)
        },
        Node::List(List { children, ordered: false, .. }) => start_ended_parent!("<ul>" children "</ul>"),
        Node::List(List { children, start: Some(s), .. }) => start_ended_parent!("<ol start=\"{s}\">" children "</ol>"),
        Node::InlineCode(InlineCode { value, .. }) => start_ended_value!("<code>" value "</code>"),
        Node::BlockQuote(BlockQuote { children, .. }) => start_ended_parent!("<blockquote>" children "</blockquote>"),
        Node::Delete(Delete { children, .. }) => start_ended_parent!("<s>" children "</s>"),
        Node::Emphasis(Emphasis { children, .. }) => start_ended_parent!("<i>" children "</i>"),
        Node::Html(Html { value, .. }) => start_ended_value!("" value ""),
        Node::Image(Image { alt, url, .. }) => start_ended_value!("<img src=\"{url}\" alt=\"" alt "\">"),
        Node::Link(Link { children, url, .. }) => start_ended_parent!("<a href=\"{url}\">" children "</a>"),
        Node::Strong(Strong { children, .. }) => start_ended_parent!("<b>" children "</b>"),
        Node::Text(Text { value, .. }) => start_ended_value!("" value ""),
        Node::Code(Code { value, lang: Some(lang), .. }) => start_ended_value!("<pre><code class=\"language-{lang}\">" value "</code></pre>"),
        Node::Code(Code { value, .. }) => start_ended_value!("<pre><code>" value "</code></pre>"),
        Node::Heading(Heading { children, depth, .. }) => start_ended_parent!("<h{depth}>" children "</h{depth}>"),
        Node::ListItem(ListItem { children, checked: Some(true), .. }) => start_ended_parent!(
            "<input type=\"checkbox\" checked disabled=\"disabled\"><li>"
            children
            "</li>"
        ),
        Node::ListItem(ListItem { children, checked: Some(_), .. }) => start_ended_parent!(
            "<input type=\"checkbox\" disabled=\"disabled\"><li>"
            children
            "</li>"
        ),
        Node::ListItem(ListItem { children, checked: None, .. }) => start_ended_parent!("<li>" children "</li>"),
        Node::Paragraph(Paragraph { children, .. }) => start_ended_parent!("" children "<br>"),
        _ => todo!("{ast:?}"),
    }
}
