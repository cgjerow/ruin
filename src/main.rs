#[macro_use]
mod debug;
mod engine;
mod graphics;
mod lua_scriptor;
mod texture;

use engine::{Engine, EngineConfig};
use mlua::prelude::*;

use crate::lua_scriptor::LuaScriptor;

fn load_engine_config() -> EngineConfig {
    let mut scriptor = LuaScriptor::new(Lua::new());
    let config_table = scriptor.execute("setup");
    let fps: String = config_table.get("fps").unwrap_or("auto".to_string());
    let debug_enabled: bool = config_table.get("debug_enabled").unwrap_or(false);
    return EngineConfig { fps, debug_enabled };
}

fn main() -> anyhow::Result<()> {
    Engine::run_event_loop(load_engine_config())
}
