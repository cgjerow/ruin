use std::fs;

use mlua::prelude::*;

use crate::components_systems::physics_2d::CollisionInfo;

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
    pub lua: Lua,
}

impl LuaExtendedExecutor {
    pub fn new(script: &str) -> Self {
        let lua = Lua::new();
        let lua_path = "./src/scripts/?.lua";
        let code = format!(
            r#"
            package.path = package.path .. ";{}"
            "#,
            lua_path
        );
        let _ = lua.load(&code).exec();
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

    pub fn rust_collisions_to_lua_2d(
        &self,
        collisions: Vec<CollisionInfo>,
    ) -> Result<LuaTable, mlua::Error> {
        let lua_table = self.lua.create_table()?;

        for (i, col) in collisions.iter().enumerate() {
            let entry = self.lua.create_table()?;

            entry.set("a", col.entity_a.0)?;
            entry.set("b", col.entity_b.0)?;
            entry.set(
                "next_pos_a",
                self.lua.create_sequence_from(col.next_pos_a.to_vec())?,
            )?;
            entry.set(
                "next_pos_b",
                self.lua.create_sequence_from(col.next_pos_b.to_vec())?,
            )?;
            entry.set(
                "a_size",
                self.lua.create_sequence_from(col.a_size.to_vec())?,
            )?;
            entry.set(
                "b_size",
                self.lua.create_sequence_from(col.b_size.to_vec())?,
            )?;
            entry.set(
                "velocity_a",
                self.lua.create_sequence_from(col.velocity_a.to_vec())?,
            )?;
            entry.set(
                "velocity_b",
                self.lua.create_sequence_from(col.velocity_b.to_vec())?,
            )?;
            entry.set(
                "normal",
                self.lua.create_sequence_from(col.normal.to_vec())?,
            )?;

            lua_table.set(i + 1, entry)?;
        }

        Ok(lua_table)
    }

    pub fn table_to_vec_8(table: LuaTable) -> [bool; 8] {
        [
            table.get::<bool>(0).unwrap_or(false),
            table.get::<bool>(1).unwrap_or(false),
            table.get::<bool>(2).unwrap_or(false),
            table.get::<bool>(3).unwrap_or(false),
            table.get::<bool>(4).unwrap_or(false),
            table.get::<bool>(5).unwrap_or(false),
            table.get::<bool>(6).unwrap_or(false),
            table.get::<bool>(7).unwrap_or(false),
        ]
    }

    #[allow(unused)]
    pub fn pretty_print_table(table: LuaTable, indent: usize) -> Result<String, mlua::Error> {
        let mut output = String::new();
        let pad = "  ".repeat(indent);

        for pair in table.pairs::<LuaValue, LuaValue>() {
            let (key, value) = pair?;

            let key_str = match &key {
                LuaValue::String(s) => s.to_str()?.to_string(),
                LuaValue::Integer(i) => i.to_string(),
                LuaValue::Number(n) => n.to_string(),
                LuaValue::Boolean(b) => b.to_string(),
                other => format!("{:?}", other),
            };

            let value_str = match &value {
                LuaValue::Table(t) => Self::pretty_print_table(t.clone(), indent + 1)?,
                LuaValue::String(s) => format!("{:?}", s.to_str()?),
                LuaValue::Boolean(b) => b.to_string(),
                LuaValue::Integer(i) => i.to_string(),
                LuaValue::Number(n) => n.to_string(),
                LuaValue::Nil => "nil".to_string(),
                other => format!("{:?}", other),
            };

            output.push_str(&format!("{}{} = {}\n", pad, key_str, value_str));
        }

        Ok(output)
    }
}
