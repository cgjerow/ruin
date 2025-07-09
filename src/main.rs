#[macro_use]
mod debug;
mod camera;
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
    let width: f32 = config_table.get("width").unwrap_or(1000.0);
    let height: f32 = config_table.get("width").unwrap_or(1000.0);
    let debug_enabled: bool = config_table.get("debug_enabled").unwrap_or(false);
    return EngineConfig {
        fps,
        debug_enabled,
        width,
        height,
    };
}

fn main() -> anyhow::Result<()> {
    Engine::run_event_loop(load_engine_config())
}
