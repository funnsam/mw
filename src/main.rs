use std::{path::*, fs::*};

mod compiler;

fn main() {
    let mut options = compiler::CompileOptions {
        md_options: markdown::ParseOptions::gfm(),
    };

    options.md_options.constructs.frontmatter = true;

    search(std::env::current_dir().unwrap(), &options);
}

fn search(path: PathBuf, options: &compiler::CompileOptions) {
    for f in std::fs::read_dir(path).unwrap() {
        if f.as_ref().unwrap().metadata().unwrap().is_dir() {
            search(f.unwrap().path(), options);
        } else if f.as_ref().unwrap().path().extension().map_or(false, |ext| ext == "md") {
            let mut output_path = f.as_ref().unwrap().path();
            output_path.set_extension("html");

            compiler::compile(output_path, &String::from_utf8(read(f.unwrap().path()).unwrap()).unwrap(), &options);
        }
    }
}
