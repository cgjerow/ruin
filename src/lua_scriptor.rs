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
