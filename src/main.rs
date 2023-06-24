#![feature(variant_count)]
#![allow(cast_ref_to_mut)]

mod markdown;
mod placeholder;

use markdown::html::Opts;
use walkdir::WalkDir;
use rayon::prelude::*;

fn main() {
    let preamble = std::fs::read_to_string("preamble.tex").ok();
    let global_ini = std::fs::read_to_string("markdown/_global.ini").unwrap().replace("\r\n", "\n");
    let gopts = Opts::load_global(global_ini, preamble);

    let _ = fs_extra::dir::remove("website");

    WalkDir::new("markdown").into_iter().par_bridge().filter_map(|file| file.ok()).for_each(|file| {
        if file.metadata().unwrap().is_file() {
            let ex = file.path().extension();
            if ex.is_none() { return; }
            match ex.unwrap().to_str().unwrap() {
                "md" => {
                    let tar_file = format!("website/{}", file.path().strip_prefix("markdown").unwrap().with_extension("html").to_str().unwrap());

                    eprintln!("\x1b[1;32mCompiling\x1b[0m\t{} -> {}", file.path().display(), tar_file);

                    let opts = std::fs::read_to_string(file.path()).ok();
                    let opts = Opts::load_ini(opts, &gopts);

                    let f = std::fs::read_to_string(file.path()).unwrap().replace("\r\n", "\n");

                    let html = markdown::html::to_html(&f, &opts);
                    let _ = std::fs::create_dir_all(std::path::Path::new(&tar_file).parent().unwrap().to_str().unwrap());
                    let _ = std::fs::write(tar_file, html).unwrap();
                },
                "html" => {
                    let tar = format!("website/{}", file.path().strip_prefix("markdown").unwrap().to_str().unwrap());
                    eprintln!("\x1b[1;32mCopying\x1b[0m\t\t{} -> {}", file.path().display(), tar);

                    let _ = fs_extra::file::copy(file.path().to_str().unwrap(), tar, &fs_extra::file::CopyOptions::default());
                },
                _ => ()
            }
        } else {
            match file.path().file_name().unwrap().to_str().unwrap() {
                "res" => {
                    let tar = format!("website/{}", file.path().strip_prefix("markdown").unwrap().to_str().unwrap());
                    eprintln!("\x1b[1;32mCopying\x1b[0m\t\t{} -> {}", file.path().display(), tar);

                    let _ = fs_extra::dir::copy(file.path().to_str().unwrap(), tar, &fs_extra::dir::CopyOptions::default().copy_inside(true));
                },
                _ => ()
            }
        }
    })
}
