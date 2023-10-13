#![feature(variant_count)]
#![allow(cast_ref_to_mut)]

mod markdown;
mod placeholder;

use markdown::html::Opts;
use walkdir::WalkDir;
use rayon::prelude::*;
use std::process::exit;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::fs::*;

fn main() {
    let preamble = read_to_string("preamble.tex").ok();
    let global_ini = read_to_string("markdown/_global.cfg").unwrap().replace("\r\n", "\n");
    let gopts = Opts::load_global(global_ini, preamble);

    let mut templates = HashMap::new();
    for file in WalkDir::new("templates").into_iter().filter_map(|file| file.ok()).filter(|file| file.metadata().unwrap().is_file()) {
        let tem = read_to_string(file.path()).unwrap().replace("\r\n", "\n");
        let hash = hash_file(&tem, file.clone());
        templates.insert(file.file_name().to_str().unwrap().to_string(), (tem, hash));
    }

    WalkDir::new("markdown").into_iter().par_bridge().filter_map(|file| file.ok()).for_each(|file| {
        if file.metadata().unwrap().is_file() {
            let ex = file.path().extension();
            if ex.is_none() { return; }
            match ex.unwrap().to_str().unwrap() {
                "md" => {
                    let tar_file = format!("website/{}", file.path().strip_prefix("markdown").unwrap().with_extension("html").to_str().unwrap());

                    eprintln!("\x1b[1;32mCompiling\x1b[0m\t{} -> {}", file.path().display(), tar_file);

                    let opts_path = file.path().with_extension("cfg");
                    let opts_path = opts_path.to_str().unwrap();
                    let opts = read_to_string(file.path().with_extension("cfg").to_str().unwrap()).ok();
                    let opts_hasheq = hash_path(&opts, opts_path);
                    let opts = Opts::load_ini(opts, &gopts);

                    let md = read_to_string(file.path()).unwrap().replace("\r\n", "\n");
                    let md_hasheq = hash_file(&md, file);
                    let (tem, tem_hasheq) = templates.get(&opts.template).unwrap();

                    if !(std::path::Path::new(&tar_file).exists() && opts_hasheq && md_hasheq && *tem_hasheq) {
                        let html = markdown::html::to_html(&md, &tem, &opts, &gopts);
                        let html = match html {
                            Ok(html) => html,
                            Err(err) => {
                                eprintln!("\x1b[1;31mError:\x1b[0m template error: {err}");
                                exit(1);
                            },
                        };

                        create_dir_all(std::path::Path::new(&tar_file).parent().unwrap().to_str().unwrap()).unwrap();
                        write(tar_file, html).unwrap();
                    }
                },
                "html" => {
                    let tar_file = format!("website/{}", file.path().strip_prefix("markdown").unwrap().to_str().unwrap());
                    eprintln!("\x1b[1;32mCopying\x1b[0m\t\t{} -> {}", file.path().display(), tar_file);

                    let opts_path = file.path().with_extension("cfg");
                    let opts_path = opts_path.to_str().unwrap();
                    let opts = read_to_string(file.path().with_extension("cfg").to_str().unwrap()).ok();
                    let opts_hasheq = hash_path(&opts, opts_path);
                    let opts = Opts::load_ini(opts, &gopts);

                    let src = read_to_string(file.path()).unwrap().replace("\r\n", "\n");
                    let src_hasheq = hash_file(&src, file);
                    let (tem, tem_hasheq) = templates.get(&opts.template).unwrap();

                    if !(std::path::Path::new(&tar_file).exists() && opts_hasheq && src_hasheq && *tem_hasheq) {
                        let html = placeholder::placeholder_process(&tem, &opts.ini, &gopts.ini, &src);
                        let html = match html {
                            Ok(html) => html,
                            Err(err) => {
                                eprintln!("\x1b[1;31mError:\x1b[0m template error: {err}");
                                exit(1);
                            },
                        };

                        create_dir_all(std::path::Path::new(&tar_file).parent().unwrap().to_str().unwrap()).unwrap();
                        write(tar_file, html).unwrap();
                    }
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

fn hash<A: Hash>(file: &A) -> u64 {
    let mut hasher = DefaultHasher::new();
    file.hash(&mut hasher);
    hasher.finish()
}

fn get_hash(rel: &str) -> Option<u64> {
    let hash_file = format!("cache/{rel}");
    let f = read(&hash_file).ok()?;
    Some(u64::from_le_bytes(f.try_into().ok()?))
}

fn set_hash(rel: &str, hash: u64) {
    let hash_file = format!("cache/{rel}");
    let _ = create_dir_all(std::path::Path::new(&hash_file).parent().unwrap().to_str().unwrap());
    let _ = write(&hash_file, &hash.to_le_bytes()).unwrap();
}

fn hash_file<A: Hash>(cont: &A, file: walkdir::DirEntry) -> bool {
    let rel = if file.path().is_absolute() {
        file.path().strip_prefix(std::env::current_dir().unwrap()).unwrap().to_str().unwrap()
    } else {
        file.path().to_str().unwrap()
    };

    hash_path(cont, rel)
}

fn hash_path<A: Hash>(cont: &A, rel: &str) -> bool {
    let hash = hash(cont);
    let old_hash = get_hash(rel).unwrap_or(0);
    set_hash(rel, hash);
    hash == old_hash
}
