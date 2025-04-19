use crate::log::LoggedUnwrap;
use markdown::{mdast::*, *};

const HTML_ALIGNMENTS: [&'static str; 4] = [
    r#"style="text-align:left""#,
    r#"style="text-align:right""#,
    r#"style="text-align:center""#,
    "",
];

pub struct CompileOptions {
    pub md_options: ParseOptions,
}

pub fn compile(content: &str, options: &CompileOptions) -> (String, toml::Table) {
    let ast = to_mdast(content, &options.md_options).logged_unwrap();

    let mut html = String::new();
    let popt = to_html(&ast, &mut html).logged_unwrap();
    (html, popt)
}

macro_rules! _start_ended_parent {
    ($start: tt $children: tt $end: tt => $acc: tt) => {{
        *$acc += &format!($start);

        for c in $children.iter() {
            to_html(c, $acc);
        }

        *$acc += &format!($end);
    }};
}
macro_rules! _start_ended_value {
    ($start: tt $value: tt $end: tt => $acc: tt) => {{
        *$acc += &format!($start);
        *$acc += &html_escape::encode_text($value);
        *$acc += &format!($end);
    }};
}

pub fn to_html(ast: &Node, acc: &mut String) -> Option<toml::Table> {
    macro_rules! start_ended_parent {
        ($start: tt $children: tt $end: tt) => {{
            _start_ended_parent!($start $children $end => acc);
            None
        }};
    }
    macro_rules! start_ended_value {
        ($start: tt $value: tt $end: tt) => {{
            _start_ended_value!($start $value $end => acc);
            None
        }};
    }

    match ast {
        Node::Root(Root { children, .. }) => {
            let mut opt = toml::Table::new();

            for c in children.iter() {
                to_html(c, acc).map(|t| opt = t);
            }

            Some(opt)
        }
        Node::Toml(Toml { value, .. }) => Some(value.parse().logged_unwrap()),
        Node::Blockquote(Blockquote { children, .. }) => {
            start_ended_parent!("<blockquote>" children "</blockquote>")
        }
        Node::List(List {
            children,
            ordered: false,
            ..
        }) => start_ended_parent!("<ul>" children "</ul>"),
        Node::List(List {
            children,
            start: Some(s),
            ..
        }) => start_ended_parent!("<ol start=\"{s}\">" children "</ol>"),
        Node::InlineCode(InlineCode { value, .. }) => start_ended_value!("<code>" value "</code>"),
        Node::InlineMath(InlineMath { value, .. }) => {
            let opts = katex::Opts::builder()
                .display_mode(false)
                .output_type(katex::OutputType::Html)
                .throw_on_error(true)
                .trust(true)
                .build()
                .logged_unwrap();
            *acc += &katex::render_with_opts(value, opts).logged_unwrap();

            None
        },
        Node::Delete(Delete { children, .. }) => start_ended_parent!("<s>" children "</s>"),
        Node::Emphasis(Emphasis { children, .. }) => start_ended_parent!("<i>" children "</i>"),
        Node::Html(Html { value, .. }) => {
            *acc += &value;
            None
        },
        Node::Image(Image { alt, url, .. }) => {
            start_ended_value!("<img src=\"{url}\" alt=\"" alt "\">")
        }
        Node::Link(Link { children, url, .. }) => {
            start_ended_parent!("<a href=\"{url}\">" children "</a>")
        }
        Node::Strong(Strong { children, .. }) => start_ended_parent!("<b>" children "</b>"),
        Node::Text(Text { value, .. }) => {
            *acc += &html_escape::encode_text(&emojicons_2021::EmojiFormatter(value).to_string());
            None
        },
        Node::Code(Code {
            value,
            lang: Some(lang),
            ..
        }) => start_ended_value!("<pre><code class=\"language-{lang}\">" value "</code></pre>"),
        Node::Code(Code { value, .. }) => start_ended_value!("<pre><code>" value "</code></pre>"),
        Node::Math(Math { value, .. }) => {
            let opts = katex::Opts::builder()
                .output_type(katex::OutputType::Html)
                .display_mode(true)
                .throw_on_error(true)
                .trust(true)
                .build()
                .logged_unwrap();
            *acc += &katex::render_with_opts(value, opts).logged_unwrap();

            None
        },
        Node::Heading(Heading {
            children, depth, ..
        }) => start_ended_parent!("<h{depth}>" children "</h{depth}>"),
        Node::Table(Table {
            children, align, ..
        }) => {
            *acc += "<table>";

            for (i, c) in children.iter().enumerate() {
                table_rows_to_html(c, acc, i == 0, &align);
            }

            *acc += "</table>";

            None
        }
        Node::ListItem(ListItem {
            children,
            checked: Some(true),
            ..
        }) => start_ended_parent!(
            "<input type=\"checkbox\" checked disabled=\"disabled\"><li>"
            children
            "</li>"
        ),
        Node::ListItem(ListItem {
            children,
            checked: Some(_),
            ..
        }) => start_ended_parent!(
            "<input type=\"checkbox\" disabled=\"disabled\"><li>"
            children
            "</li>"
        ),
        Node::ListItem(ListItem {
            children,
            checked: None,
            ..
        }) => start_ended_parent!("<li>" children "</li>"),
        Node::Paragraph(Paragraph { children, .. }) => start_ended_parent!("<p>" children "</p>"),
        _ => todo!("{ast:?}"),
    }
}

fn table_rows_to_html(ast: &Node, acc: &mut String, is_first_row: bool, alignment: &[AlignKind]) {
    match ast {
        Node::TableRow(TableRow { children, .. }) => {
            *acc += "<tr>";

            for (i, c) in children.iter().enumerate() {
                table_cells_to_html(c, acc, is_first_row, HTML_ALIGNMENTS[alignment[i] as usize]);
            }

            *acc += "</tr>";
        }
        _ => {
            to_html(ast, acc);
        }
    }
}

fn table_cells_to_html(ast: &Node, acc: &mut String, is_first_row: bool, alignment: &str) {
    match ast {
        Node::TableCell(TableCell { children, .. }) if is_first_row => {
            _start_ended_parent!("<th {alignment}>" children "</th>" => acc)
        }
        Node::TableCell(TableCell { children, .. }) => {
            _start_ended_parent!("<td {alignment}>" children "</td>" => acc)
        }
        _ => {
            to_html(ast, acc);
        }
    }
}
