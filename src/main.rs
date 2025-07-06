use mlua::prelude::*;
use once_cell::sync::OnceCell;
use std::fs;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

#[derive(Debug)]
struct Config {
    fps: Duration,
    target_rate: Duration,
}

static CONFIG: OnceCell<Config> = OnceCell::new();

fn main() {
    const SAFETY_MAX_FOR_DEV: u16 = 100;
    let mut count: u16 = 0;

    setup();

    println!("Starting Game Loop");
    let mut last_frame: Instant = Instant::now();
    loop {
        update(last_frame);
        draw();
        last_frame = Instant::now();
        count += 1;
        if count > SAFETY_MAX_FOR_DEV {
            break;
        }
    }

    cleanup()
}

fn setup() {
    println!("setup one time");
    lua_setup();
}

fn lua_setup() {
    let lua = Lua::new();

    let lua_code =
        fs::read_to_string("src/lua-scripts/setup.lua").expect("Failed to read setup.lua");
    lua.load(&lua_code)
        .exec()
        .expect("Failed to execute setup.lua");
    let globals = lua.globals();
    let setup_fn: LuaFunction = globals.get("setup").expect("Failed to setup Lua");
    let config_table: LuaTable = setup_fn.call(()).expect("Setup call failed");

    let _ = CONFIG.set(Config {
        fps: Duration::from_millis(config_table.get("fps").unwrap_or(60)),
        target_rate: Duration::from_millis(1000 / (config_table.get("fps").unwrap_or(60))),
    });
}

fn draw() {
    println!("draw")
}

fn update(last_frame: Instant) {
    sleep(CONFIG.get().unwrap().target_rate - last_frame.elapsed());
    println!(
        "update: time since last frame {}",
        last_frame.elapsed().as_millis()
    )
}

fn cleanup() {
    println!("clean!")
}
