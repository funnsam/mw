#![feature(variant_count)]
#![allow(cast_ref_to_mut)]

mod markdown;

fn main() {
    let file = std::fs::read_to_string("test.md").unwrap().replace("\r\n", "\n");
    println!("{}", markdown::html::to_html(&file));
}
