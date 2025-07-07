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
        let lua_code = fs::read_to_string(format!("src/scripts/{}.lua", script))
            .expect(&format!("Failed to read {}.lua", script));
        self.lua
            .load(&lua_code)
            .exec()
            .expect(&format!("Failed to load script: {}", script));

        let globals = self.lua.globals();
        let main_fn: LuaFunction = globals.get("main").expect(&format!(
            "Function `main` not defined for script: {}",
            script
        ));

        return main_fn
            .call(())
            .expect(&format!("Failed to execute script: {}", script));
    }
}
