use std::fs;

use mlua::prelude::*;

pub struct LuaScriptor {
    lua: Lua,
}

impl LuaScriptor {
    pub fn new(lua: Lua) -> Self {
        Self { lua }
    }

    pub fn execute(&mut self, script: &str) -> LuaTable {
        // by convention all lua scripts will be in src/scripts/
        let lua_code = fs::read_to_string(format!("src/scripts/{}.lua", script))
            .expect(&format!("Failed to read {}.lua", script));
        self.lua
            .load(&lua_code)
            .exec()
            .expect(&format!("Failed to load script: {}.lua", script));

        let globals = self.lua.globals();
        let main_fn: LuaFunction = globals.get("main").expect(&format!(
            "Function `main` not defined for script: {}.lua",
            script
        ));

        return main_fn
            .call(())
            .expect(&format!("Failed to execute script: {}.lua", script));
    }
}

pub struct LuaExtendedExecutor {
    lua: Lua,
}

impl LuaExtendedExecutor {
    pub fn new(script: &str) -> Self {
        let lua = Lua::new();
        let path = format!("./src/scripts/{}.lua", script);
        let contents = fs::read_to_string(&path).expect("Unable to read Lua script file");
        lua.load(&contents)
            .exec()
            .expect("Unable to execute main.lua");
        Self { lua }
    }

    pub fn create_table(&self) -> mlua::Table {
        return self.lua.create_table().unwrap();
    }

    pub fn get_function(&self, method: &str) -> mlua::Function {
        let lua_func: mlua::Function = self
            .lua
            .globals()
            .get(method)
            .expect("function does not exist");
        return lua_func;
    }
}
