#[macro_use]
use mlua::prelude::*;
use ruin_camera::{Camera2DConfig, CameraOption, Dimensions};
use ruin_engine::{Engine, EngineConfig};
use ruin_lua_runtime::{LuaExtendedExecutor, LuaScriptor};
use winit::event_loop::EventLoop;

fn load_engine_config() -> EngineConfig {
    let mut scriptor = LuaScriptor::new(Lua::new());
    let config_table = scriptor.execute("setup");
    let fps: String = config_table.get("fps").unwrap_or("auto".to_string());
    let virtual_resolution_width: u32 = config_table.get("virtual_resolution_width").unwrap_or(320);
    let virtual_resolution_height: u32 =
        config_table.get("virtual_resolution_height").unwrap_or(180);
    let window_width: u32 = config_table.get("window_width").unwrap_or(1000);
    let window_height: u32 = config_table.get("window_height").unwrap_or(1000);
    let camera2d_config: mlua::Table = config_table
        .get("camera_config")
        .unwrap_or(scriptor.lua.create_table().unwrap());
    let debug_enabled: bool = config_table.get("debug_enabled").unwrap_or(false);
    return EngineConfig {
        fps,
        debug_enabled,
        window_width,
        window_height,
        virtual_resolution_width,
        virtual_resolution_height,
        dimensions: Dimensions::Two,
        camera: CameraOption::Follow,
        camera2d_config: Camera2DConfig {
            zoom: camera2d_config.get("zoom").unwrap_or(15.0),
            initial_position: [
                camera2d_config.get("initial_pos_x").unwrap_or(0.0),
                camera2d_config.get("initial_pos_y").unwrap_or(0.0),
            ],
            look_ahead_smooth_factor: camera2d_config
                .get("look_ahead_smooth_factor")
                .unwrap_or(10.0),
            look_ahead_distance: camera2d_config.get("look_ahead_distance").unwrap_or(10.0),
            look_ahead_lerp_speed: camera2d_config.get("look_ahead_lerp_speed").unwrap_or(2.0),
            screen_width: window_width as f32,
            screen_height: window_height as f32,
        },
    };
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    let lua = LuaExtendedExecutor::new("ruin");
    let mut app = Engine::new(load_engine_config(), lua);
    event_loop.run_app(&mut app)?;

    return Ok(());
}
