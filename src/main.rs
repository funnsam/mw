use std::{fs::*, path::*, str::FromStr};

mod args;
mod compiler;
#[macro_use]
mod log;
use clap::Parser;
use log::LoggedUnwrap;
use mlua::Function;

fn get_options() -> compiler::CompileOptions {
    let mut options = compiler::CompileOptions {
        md_options: markdown::ParseOptions::gfm(),
    };

    options.md_options.constructs.frontmatter = true;
    options.md_options.constructs.math_text = true;
    options.md_options.constructs.math_flow = true;
    options.md_options.constructs.html_text = true;
    options.md_options.constructs.html_flow = true;

    options
}

fn init_lua(lua: &mlua::Lua) {
    let globals = lua.globals();

    let config = &read_to_string("./config.toml")
        .logged_unwrap()
        .parse::<toml::Table>()
        .logged_unwrap();
    globals.set("config", toml_table_to_lua_table(config, &lua)).logged_unwrap();

    macro_rules! lua_fn {
        ($id: ident = $fn: expr) => {
            let func = lua.create_function($fn).logged_unwrap();
            globals.set(stringify!($id), func).logged_unwrap();
        };
    }

    lua_fn!(path_parent = |_lua, path: String| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .parent()
                .map(|p| p.to_str().logged_unwrap().to_string())
        )
    });

    lua_fn!(path_relative = |_lua, path: String| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .strip_prefix(std::env::current_dir().logged_unwrap())
                .map(|p| p.to_str().logged_unwrap().to_string())
                .ok()
        )
    });

    lua_fn!(path_relative_to = |_lua, (path, root): (String, String)| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .strip_prefix(&root)
                .map(|p| p.to_str().logged_unwrap().to_string())
                .ok()
        )
    });

    lua_fn!(path_filename = |_lua, path: String| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .file_name()
                .map(|p| p.to_str().logged_unwrap().to_string())
        )
    });

    lua_fn!(path_extension = |_lua, path: String| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .extension()
                .map(|p| p.to_str().logged_unwrap().to_string())
        )
    });

    lua_fn!(path_join = |_lua, (base, path): (String, String)| {
        Ok(
            PathBuf::from_str(&base).logged_unwrap()
                .join(&path)
                .to_str().logged_unwrap()
                .to_string()
        )
    });

    lua_fn!(path_with_filename = |_lua, (path, filename): (String, String)| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .with_file_name(&filename)
                .to_str().logged_unwrap()
                .to_string()
        )
    });

    lua_fn!(path_with_extension = |_lua, (path, ext): (String, String)| {
        Ok(
            PathBuf::from_str(&path).logged_unwrap()
                .with_extension(&ext)
                .to_str().logged_unwrap()
                .to_string()
        )
    });

    lua_fn!(search_in = |lua, (md_path, out_path, depth): (String, String, Option<usize>)| {
        let mut opts = vec![];

        search(
            PathBuf::from_str(&md_path).logged_unwrap(),
            PathBuf::from_str(&out_path).logged_unwrap(),
            lua,
            depth.unwrap_or(0),
            &mut opts,
        );

        Ok(opts)
    });


    lua.load(read("./postprocess.lua").logged_unwrap())
        .set_name("postprocess.lua")
        .exec()
        .logged_unwrap();
}

fn main() {
    let mode = args::Mode::parse();

    let lua = unsafe { mlua::Lua::unsafe_new() };
    init_lua(&lua);
    let globals = lua.globals();

    let project_base = std::env::current_dir().logged_unwrap();
    let pages_base = project_base.join("pages");
    let output_base = project_base.join("output");
    globals.set("project_base", project_base.to_str().logged_unwrap()).logged_unwrap();
    globals.set("pages_base", pages_base.to_str().logged_unwrap()).logged_unwrap();
    globals.set("output_base", output_base.to_str().logged_unwrap()).logged_unwrap();

    match mode {
        args::Mode::Build => {
            search(
                pages_base,
                output_base,
                &lua,
                0,
                &mut vec![],
            );
        },
        #[cfg(feature = "watch")]
        args::Mode::Watch => {
            use notify::*;

            search(
                pages_base.clone(),
                output_base.clone(),
                &lua,
                0,
                &mut vec![],
            );

            let (tx, rx) = std::sync::mpsc::channel::<Result<Event>>();

            let mut watcher = recommended_watcher(tx).logged_unwrap();
            watcher.watch(&pages_base, RecursiveMode::Recursive).logged_unwrap();

            for event in rx {
                if let Ok(Event { kind: EventKind::Create(..) | EventKind::Modify(..), paths, .. }) = event {
                    // TODO: make it so that it only generate for changed files
                    search(
                        pages_base.clone(),
                        output_base.clone(),
                        &lua,
                        0,
                        &mut vec![],
                    );
                    info!("regenerated website for {} path(s)", paths.len());
                }
            }
        },
    }
}

fn search<'a>(
    pages_path: PathBuf,
    output_path: PathBuf,
    lua: &'a mlua::Lua,
    depth: usize,
    pages: &mut Vec<mlua::Table<'a>>,
) {
    for f in std::fs::read_dir(pages_path).logged_unwrap() {
        let p = f.as_ref().logged_unwrap().path();
        let last = p
            .file_name()
            .logged_unwrap()
            .to_str()
            .logged_unwrap()
            .to_string();

        if f.as_ref()
            .logged_unwrap()
            .metadata()
            .logged_unwrap()
            .is_dir()
        {
            search(p, output_path.join(last), lua, depth + 1, pages);
        } else {
            match p.extension().and_then(|a| a.to_str()) {
                Some("md") => {
                    let out = output_path.clone().join(last).with_extension("html");

                    let (body, raw_opts) = compiler::compile(
                        &String::from_utf8(read(f.logged_unwrap().path()).logged_unwrap()).logged_unwrap(),
                        &get_options(),
                    );
                    let opts = toml_table_to_lua_table(&raw_opts, lua);

                    let result = lua.globals().get::<_, Function>("generate_final_html").logged_unwrap().call::<_, String>((
                        p.to_str().logged_unwrap(),
                        out.to_str().logged_unwrap(),
                        depth,
                        body,
                        &opts,
                    )).logged_unwrap();

                    let page = lua.create_table().logged_unwrap();
                    page.set("md_path", p.to_str().logged_unwrap()).logged_unwrap();
                    page.set("out_path", out.to_str().logged_unwrap()).logged_unwrap();
                    page.set("options", opts).logged_unwrap();
                    pages.push(page);

                    create_dir_all(&output_path).logged_unwrap();
                    write(out, result).logged_unwrap();
                },
                Some(_) if p.file_name().and_then(|a| a.to_str()).unwrap_or("").contains(".copy.") => {
                    create_dir_all(&output_path).logged_unwrap();
                    write(output_path.join(last.replace(".copy", "")), read(p).logged_unwrap()).logged_unwrap();
                }
                Some("copy") => {
                    create_dir_all(&output_path).logged_unwrap();
                    write(output_path.join(last).with_extension(""), read(p).logged_unwrap()).logged_unwrap();
                }
                _ => warn!("found unknown type of file (`{}`) (note: use `.copy` before file extension to copy the file to output directory.)", p.display()),
            }
        }
    }
}

fn toml_table_to_lua_table<'lua>(t: &toml::Table, lua: &'lua mlua::Lua) -> mlua::Table<'lua> {
    let lt = lua.create_table().logged_unwrap();

    for (k, v) in t.iter() {
        lt.set(k.as_str(), toml_value_to_lua_value(v, lua))
            .logged_unwrap();
    }

    lt
}

fn toml_array_to_lua_table<'lua>(
    a: &toml::value::Array,
    lua: &'lua mlua::Lua,
) -> mlua::Table<'lua> {
    let lt = lua.create_table().logged_unwrap();

    for (i, v) in a.iter().enumerate() {
        lt.set(i + 1, toml_value_to_lua_value(v, lua))
            .logged_unwrap();
    }

    lt
}

fn toml_value_to_lua_value<'lua>(v: &toml::Value, lua: &'lua mlua::Lua) -> mlua::Value<'lua> {
    use mlua::IntoLua;
    match v {
        toml::Value::Float(f) => (*f).into_lua(lua),
        toml::Value::Array(a) => toml_array_to_lua_table(a, lua).into_lua(lua),
        toml::Value::Table(t) => toml_table_to_lua_table(t, lua).into_lua(lua),
        toml::Value::String(s) => s.as_str().into_lua(lua),
        toml::Value::Integer(i) => (*i).into_lua(lua),
        toml::Value::Boolean(b) => (*b).into_lua(lua),
        toml::Value::Datetime(d) => format!("{d}").as_str().into_lua(lua),
    }
    .logged_unwrap()
}

// LIGHT:
// Workspace structure:
// Root
// ├─ pages
// │  └─ xx.md or .copy.xxx
// ├─ config.toml: config file passed into lua
// ├─ postprocess.lua: final html generation
// └─ output
//    └─ output.html s
