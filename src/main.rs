use std::{fs::*, path::*};

mod compiler;
#[macro_use]
mod log;
use log::LoggedUnwrap;

fn main() {
    let mut options = compiler::CompileOptions {
        md_options: markdown::ParseOptions::gfm(),
    };

    options.md_options.constructs.frontmatter = true;
    options.md_options.constructs.math_text = true;

    let lua = mlua::Lua::new();
    let globals = lua.globals();

    let config = &read_to_string("./config.toml")
        .logged_unwrap()
        .parse::<toml::Table>()
        .logged_unwrap();

    for (k, v) in config.iter() {
        globals
            .set(k.as_str(), toml_value_to_lua_value(v, &lua))
            .logged_unwrap();
    }

    lua.load(read("./templates.lua").logged_unwrap())
        .set_name("templates.lua")
        .exec()
        .logged_unwrap();

    search(
        std::env::current_dir().logged_unwrap().join("pages"),
        std::env::current_dir().logged_unwrap().join("output"),
        &options,
        &lua,
        &globals.get("generate_final_html").logged_unwrap(),
        0,
    );
}

fn search(
    pages_path: PathBuf,
    output_path: PathBuf,
    options: &compiler::CompileOptions,
    lua: &mlua::Lua,
    tem: &mlua::Function,
    depth: usize,
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
            search(p, output_path.join(last), options, lua, tem, depth + 1);
        } else {
            match p.extension().and_then(|a| a.to_str()) {
                Some("md") => {
                    let out = output_path.clone().join(last).with_extension("html");

                    let (body, raw_opts) = compiler::compile(
                        &String::from_utf8(read(f.logged_unwrap().path()).logged_unwrap()).logged_unwrap(),
                        &options,
                    );
                    let opts = toml_table_to_lua_table(&raw_opts, lua);

                    let result = tem.call::<_, String>((p.to_str().logged_unwrap(), depth, body, opts)).logged_unwrap();

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
// ├─ templates.lua: final html generation
// └─ output
//    └─ output.html s
