#[macro_use]
mod debug;
mod bitmaps;
mod camera_2d;
mod camera_3d;
mod components_systems;
mod engine;
mod graphics;
mod graphics_2d;
mod graphics_3d;
mod inputs;
mod lua_scriptor;
mod texture;
mod ui_canvas;
mod world;

use engine::{Engine, EngineConfig};
use mlua::prelude::*;
use winit::event_loop::EventLoop;

use crate::{
    engine::{CameraOption, Dimensions},
    lua_scriptor::LuaScriptor,
};

fn load_engine_config() -> EngineConfig {
    let mut scriptor = LuaScriptor::new(Lua::new());
    let config_table = scriptor.execute("setup");
    let fps: String = config_table.get("fps").unwrap_or("auto".to_string());
    let width: u32 = config_table.get("width").unwrap_or(1000);
    let height: u32 = config_table.get("height").unwrap_or(1000);
    let camera2d_config: mlua::Table = config_table
        .get("camera_config")
        .unwrap_or(scriptor.lua.create_table().unwrap());
    let debug_enabled: bool = config_table.get("debug_enabled").unwrap_or(false);
    return EngineConfig {
        fps,
        debug_enabled,
        width,
        height,
        dimensions: Dimensions::Two,
        camera: CameraOption::Follow,
        camera2d_config: camera_2d::camera_2d::Camera2DConfig {
            zoom: camera2d_config.get("zoom").unwrap_or(15.0),
            initial_position: [
                camera2d_config.get("initial_pos_x").unwrap_or(0.0),
                camera2d_config.get("initial_pos_y").unwrap_or(0.0),
            ],
            look_ahead_smooth_factor: camera2d_config
                .get("look_ahead_smooth_factor")
                .unwrap_or(0.3),
            look_ahead_distance: camera2d_config.get("look_ahead_distance").unwrap_or(3.0),
            look_ahead_lerp_speed: camera2d_config.get("look_ahead_lerp_speed").unwrap_or(0.1),
            screen_width: width as f32,
            screen_height: height as f32,
        },
    };
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    let lua = lua_scriptor::LuaExtendedExecutor::new("main");
    let mut app = Engine::new(load_engine_config(), lua);
    event_loop.run_app(&mut app)?;

    return Ok(());
}
