use std::collections::HashMap;

use proc_macro_hack::proc_macro_hack;
use rlua::{Result, Context, UserData, UserDataMethods, MetaMethod, Value, Table, RegistryKey};

#[proc_macro_hack]
pub use include_lua_macro::include_lua;

pub struct LuaModules {
    files: HashMap<String, String>,
    name: String,
}

impl LuaModules {
    #[doc(hidden)] // This is not a public API!
    pub fn __new(files: HashMap<String, String>, name: &str) -> LuaModules {
        LuaModules { files: files, name: name.to_string() }
    }
}

struct Searcher(LuaModules, RegistryKey);

impl UserData for Searcher {
     fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Call, |ctx, this, value: String| {
            Ok(match this.0.files.get(&value) {
                Some(source) => {
                    Value::Function(ctx.load(source)
                        .set_name(&this.0.name)?
                        .set_environment(ctx.registry_value::<Table>(&this.1)?)?
                        .into_function()?
                    )
                }
                None => Value::Nil,
            })
        });
    }
}

pub trait ContextExt<'a> {
    fn add_modules(&self, modules: LuaModules) -> Result<()>;
    fn add_modules_with_env(&self, modules: LuaModules, environment: Table<'a>) -> Result<()>;
}

impl<'a> ContextExt<'a> for Context<'a> {
    fn add_modules(&self, modules: LuaModules) -> Result<()> {
        self.add_modules_with_env(modules, self.globals())
    }

    fn add_modules_with_env(&self, modules: LuaModules, environment: Table<'a>) -> Result<()> {
        let key = self.create_registry_value(environment)?;
        let searchers: Table = self.globals().get::<_, Table>("package")?.get("searchers")?;
        searchers.set(searchers.len()? + 1, Searcher(modules, key))?;
        Ok(())
    }
}