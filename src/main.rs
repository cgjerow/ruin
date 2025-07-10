#[macro_use]
mod debug;
mod camera;
mod engine;
mod graphics;
mod lua_scriptor;
mod texture;

use engine::{Engine, EngineConfig};
use mlua::prelude::*;
use winit::event_loop::EventLoop;

use crate::lua_scriptor::LuaScriptor;

fn load_engine_config() -> EngineConfig {
    let mut scriptor = LuaScriptor::new(Lua::new());
    let config_table = scriptor.execute("setup");
    let fps: String = config_table.get("fps").unwrap_or("auto".to_string());
    let width: u32 = config_table.get("width").unwrap_or(1000);
    let height: u32 = config_table.get("width").unwrap_or(1000);
    let debug_enabled: bool = config_table.get("debug_enabled").unwrap_or(false);
    return EngineConfig {
        fps,
        debug_enabled,
        width,
        height,
    };
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    let lua = lua_scriptor::LuaExtendedExecutor::new("main");
    let mut app = Engine::new(load_engine_config(), lua);
    event_loop.run_app(&mut app)?;

    return Ok(());
}
