use mlua::prelude::*;
use once_cell::sync::OnceCell;
use std::fs;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Debug)]
struct Config {
    fps: Duration,
    target_rate: Duration,
}

static CONFIG: OnceCell<Config> = OnceCell::new();
static SAFETY_MAX_FOR_DEV: u64 = 100;

struct App {
    window: Option<Window>,
    last_frame: Instant,
    count: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            last_frame: Instant::now(),
            count: 0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        setup();
        self.last_frame = Instant::now();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        update(self.last_frame);
        draw();
        self.last_frame = Instant::now();
        self.count += 1;
        if self.count > SAFETY_MAX_FOR_DEV {
            event_loop.exit()
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        cleanup()
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Unable to open window");
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
