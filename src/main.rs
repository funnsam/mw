use std::{fs::*, path::*};

mod compiler;
#[macro_use]
mod log;

fn main() {
    let mut options = compiler::CompileOptions {
        md_options: markdown::ParseOptions::gfm(),
    };

    options.md_options.constructs.frontmatter = true;

    let lua = mlua::Lua::new();

    let titlebar = toml_table_to_lua_table(
        &read_to_string("./titlebar.toml").unwrap().parse().unwrap(),
        &lua,
    );
    lua.globals()
        .set(
            "titlebar",
            titlebar.get::<_, mlua::Table>("titlebar").unwrap(),
        )
        .unwrap();

    lua.load(read("./templates.lua").unwrap()).exec().unwrap();

    create_dir_all("output").unwrap();
    search(
        std::env::current_dir().unwrap().join("pages"),
        std::env::current_dir().unwrap().join("output"),
        &options,
        &lua,
        &lua.globals().get("generate_final_html").unwrap(),
    );
}

fn search(
    pages_path: PathBuf,
    output_path: PathBuf,
    options: &compiler::CompileOptions,
    lua: &mlua::Lua,
    tem: &mlua::Function,
) {
    for f in std::fs::read_dir(pages_path).unwrap() {
        let p = f.as_ref().unwrap().path();
        let last = p.file_name().unwrap().to_str().unwrap().to_string();

        if f.as_ref().unwrap().metadata().unwrap().is_dir() {
            search(p, output_path.join(last), options, lua, tem);
        } else {
            match p.extension().and_then(|a| a.to_str()) {
                Some("md") => {
                    let out = output_path.join(last).with_extension("html");

                    let (body, raw_opts) = compiler::compile(
                        &String::from_utf8(read(f.unwrap().path()).unwrap()).unwrap(),
                        &options,
                    );
                    let opts = toml_table_to_lua_table(&raw_opts, lua);

                    let result = tem.call::<_, String>((p.to_str().unwrap(), body, opts)).unwrap();
                    write(out, result).unwrap();
                },
                Some(_) if p.file_name().and_then(|a| a.to_str()).unwrap_or("").contains(".copy.") => {
                    write(output_path.join(last.replace(".copy", "")), read(p).unwrap()).unwrap();
                }
                Some("copy") => {
                    write(output_path.join(last).with_extension(""), read(p).unwrap()).unwrap();
                }
                _ => warn!("found unknown type of file (`{}`) (note: use `.copy` before file extension to copy the file to output directory.)", p.display()),
            }
        }
    }
}

fn toml_table_to_lua_table<'lua>(t: &toml::Table, lua: &'lua mlua::Lua) -> mlua::Table<'lua> {
    let lt = lua.create_table().unwrap();

    for (k, v) in t.iter() {
        lt.set(k.as_str(), toml_value_to_lua_value(v, lua)).unwrap();
    }

    lt
}

fn toml_array_to_lua_table<'lua>(
    a: &toml::value::Array,
    lua: &'lua mlua::Lua,
) -> mlua::Table<'lua> {
    let lt = lua.create_table().unwrap();

    for (i, v) in a.iter().enumerate() {
        lt.set(i + 1, toml_value_to_lua_value(v, lua)).unwrap();
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
    .unwrap()
}

// LIGHT:
// Workspace structure:
// Root
// ├─ pages
// │  └─ xx.md or .copy.xxx
// │
// ├─ titlebar.toml: titlebar item list
// ├─ templates.lua: final html generation
// └─ output
//    └─ output.html s
// ────────────────────────────────────────
// Lua html generation:
//  function renerate_final_html(body, options) -> String (write to html)
//  _G.titlebar (array of titlebar items from titlebar.toml)
