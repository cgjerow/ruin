#[macro_use]
mod debug;
mod engine;
mod graphics;

use std::fs;

use engine::{Engine, EngineConfig};
use mlua::prelude::*;

fn load_lua_config() -> EngineConfig {
    let lua = Lua::new();
    let lua_code = fs::read_to_string("src/scripts/setup.lua").expect("Failed to read setup.lua");
    lua.load(&lua_code)
        .exec()
        .expect("Failed to execute setup.lua");
    let globals = lua.globals();
    let setup_fn: LuaFunction = globals.get("setup").expect("Failed to setup Lua");
    let config_table: LuaTable = setup_fn.call(()).expect("Setup call failed");
    let fps: String = config_table.get("fps").unwrap_or("auto".to_string());
    let debug_enabled: bool = config_table.get("debug_enabled").unwrap_or(false);
    return EngineConfig { fps, debug_enabled };
}

fn main() -> anyhow::Result<()> {
    Engine::run_event_loop(load_lua_config())
}
