use crate::camera::camera::CameraMode;
use crate::camera::{
    Camera, CameraAction, CameraController, CameraInputMap, ThreeDimensionalCameraController,
    TwoDimensionalCameraController, UniversalCameraController,
};
use crate::game_element::{Animation, StatefulElement, VisualState};
use crate::graphics::ElementsToRender;
use crate::lua_scriptor::LuaExtendedExecutor;
use crate::texture::Texture;
use crate::{debug, graphics};
use debug::Debug;
use graphics::Graphics;
use mlua::{Result, Table};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

static SAFETY_MAX_FOR_DEV: u64 = 10000;

pub struct Engine {
    window: Option<Arc<Window>>,
    graphics: Option<Graphics>,
    count: u64,
    #[allow(dead_code)]
    fps_specified: bool,
    target_rate: Option<Duration>,
    last_frame: Instant,
    debugger: Debug,
    asset_cache: HashMap<String, Texture>,
    lua_context: LuaExtendedExecutor,
    character_cache: HashMap<String, StatefulElement>,
    width: u32,
    height: u32,
}

pub struct EngineConfig {
    pub fps: String,
    pub debug_enabled: bool,
    pub width: u32,
    pub height: u32,
}

impl Engine {
    pub fn new(config: EngineConfig, lua_executor: LuaExtendedExecutor) -> Self {
        let fps_opt = if config.fps.trim().eq_ignore_ascii_case("auto") {
            None
        } else {
            config.fps.parse::<u64>().ok()
        };

        let target_rate = fps_opt.map(|fps| Duration::from_millis(1000 / fps));

        Self {
            lua_context: lua_executor,
            window: None,
            graphics: None,
            debugger: Debug::new(config.debug_enabled),
            count: 0,
            fps_specified: fps_opt != None,
            target_rate: target_rate,
            last_frame: Instant::now() - target_rate.unwrap_or_default(),
            asset_cache: HashMap::new(),
            width: config.width,
            height: config.height,
            character_cache: HashMap::new(),
        }
    }

    fn get_texture(&mut self, path: &str) -> Texture {
        let texture = self.asset_cache.entry(path.to_string()).or_insert_with(|| {
            debug_log!(self.debugger, "Initialized asset: {}", path);
            self.graphics
                .as_mut()
                .expect("Graphics not initialized")
                .load_texture_from_path(&format!("./src/assets/{}", path))
        });

        texture.clone()
    }

    fn random_color() -> wgpu::Color {
        return wgpu::Color {
            r: rand::random(),
            b: rand::random(),
            g: rand::random(),
            a: rand::random(),
        };
    }

    fn is_targetting_fps(&self) -> bool {
        return self.fps_specified;
    }

    fn redraw(&self) {
        if self.is_targetting_fps() {
            self.window
                .as_ref()
                .expect("Window does not exist")
                .request_redraw();
        }
    }

    fn update(&mut self, dt: Duration) -> anyhow::Result<()> {
        // debug_log!(self.debugger, "Updated it? {}", true);
        let update: mlua::Function = self.lua_context.get_function("update");
        let _ = update.call::<()>(dt.as_secs_f32());

        for character in self.character_cache.values_mut() {
            character.update(dt.as_secs_f32()); // or pass actual dt if you have it
        }

        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return Ok(()),
        };

        graphics.update_camera();

        return Ok(());
    }

    fn cleanup(&mut self) {
        debug_log!(self.debugger, "Cleaned it? {}", true)
    }

    fn to_render(&self) -> ElementsToRender {
        ElementsToRender {
            elements: self.character_cache.values().cloned().collect(),
        }
    }

    fn get_window_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn create_character(&mut self, character_table: mlua::Table) {
        let id: String = character_table.get("id").unwrap_or("unknown".to_string());
        let state: VisualState = character_table
            .get("state")
            .unwrap_or("Idle".to_string())
            .into();
        let x: f32 = character_table.get("x").unwrap_or(0.0);
        let y: f32 = character_table.get("y").unwrap_or(0.0);
        let width: f32 = character_table.get("width").unwrap_or(1.0);
        let height: f32 = character_table.get("height").unwrap_or(1.0);
        let animations: mlua::Table = character_table
            .get("animations")
            .unwrap_or(self.lua_context.create_table());
        let sprite: String = character_table
            .get("sprite")
            .expect("Sprite Sheet required for Character");
        let sheet_width = character_table
            .get("sprite_sheet_width")
            .expect("Sprite Sheet Width required");
        let sheet_height = character_table
            .get("sprite_sheet_height")
            .expect("Sprite Sheet Height required");

        let texture = self.get_texture(&sprite);

        let character = StatefulElement {
            id: id.clone(),
            state,
            position: [x, y, 0.0],
            size: [width, height],
            sprite_sheet: texture,
            current_frame: 0,
            frame_timer: 0.0,
            animations: Self::table_to_map(animations, VisualState::from, |tbl| {
                Animation::from_lua_table(tbl, sheet_width, sheet_height)
            }),
        };

        self.character_cache.insert(id, character);
    }

    pub fn configure_camera(&mut self, config: mlua::Table) -> Result<()> {
        let config = LuaCameraConfig::from_lua_table(config)?;

        // Convert mode string to enum
        let mode = match config.mode.as_str() {
            "Orthographic2D" => CameraMode::Orthographic2D,
            "Perspective3D" => CameraMode::Perspective3D,
            "Universal" => CameraMode::Universal3D,
            other => {
                return Err(mlua::Error::RuntimeError(format!(
                    "Invalid camera mode: {}",
                    other
                )))
            }
        };

        // Create camera
        let camera = Camera::new(self.width, self.height, mode.clone());

        // Create input map from key bindings
        let mut input_map = CameraInputMap::new();
        for binding in config.keys {
            let key = LuaCameraConfig::parse_keycode(&binding.key)?;
            let action = LuaCameraConfig::parse_camera_action(&binding.action)?;
            input_map = input_map.insert(key, action);
        }

        // Build controller
        let controller: Box<dyn CameraController> = match mode {
            CameraMode::Orthographic2D => {
                // TODO fix inputs, take input controls
                Box::new(TwoDimensionalCameraController::new(config.speed))
            }
            CameraMode::Perspective3D => {
                // TODO fix inputs, take input controls
                Box::new(ThreeDimensionalCameraController::new(config.speed))
            }
            CameraMode::Universal3D => {
                // TODO fix inputs, take speed
                Box::new(UniversalCameraController::new(
                    config.speed,
                    config.speed,
                    input_map,
                ))
            }
        };
        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return Ok(()),
        };

        graphics.update_camera_config(camera, controller);
        Ok(())
    }

    fn table_to_map<K, V>(
        table: mlua::Table,
        key_converter: impl Fn(String) -> K,
        value_converter: impl Fn(mlua::Table) -> V,
    ) -> HashMap<K, V>
    where
        K: std::cmp::Eq + std::hash::Hash,
    {
        let mut map = HashMap::new();

        for pair in table.pairs::<String, mlua::Table>() {
            if let Ok((key, value)) = pair {
                map.insert(key_converter(key), value_converter(value));
            }
        }

        map
    }

    fn setup(&mut self) {
        let self_ptr = self as *mut Self;
        let get_window_size = self
            .lua_context
            .lua
            .create_function(move |_, ()| {
                let engine = unsafe { &*self_ptr };
                Ok(engine.get_window_size())
            })
            .expect("Could not create function");
        let create_character = self
            .lua_context
            .lua
            .create_function(move |_, character_table: mlua::Table| {
                let engine = unsafe { &mut *self_ptr };
                Ok(engine.create_character(character_table))
            })
            .expect("Could not create function");
        let configure_camera = self
            .lua_context
            .lua
            .create_function(move |_, config: mlua::Table| {
                let engine = unsafe { &mut *self_ptr };
                Ok(engine.configure_camera(config))
            })
            .expect("Could not create function");

        let lua_engine = self.lua_context.create_table();
        lua_engine
            .set("get_window_size", get_window_size)
            .expect("Could not set engine function");
        lua_engine
            .set("create_character", create_character)
            .expect("Could not set engine function");
        lua_engine
            .set("configure_camera", configure_camera)
            .expect("Could not set engine function");

        self.lua_context
            .lua
            .globals()
            .set("engine", lua_engine)
            .expect("Could not define global engine");

        let config: mlua::Table = self
            .lua_context
            .get_function("load")
            .call::<mlua::Table>({})
            .expect("Unable to load initial assets.");

        let assets = config
            .get::<mlua::Table>("assets")
            .unwrap_or_else(|_| self.lua_context.create_table());
        for asset in assets.sequence_values::<String>() {
            let asset = asset.unwrap_or("".to_string());
            if asset != "" {
                let _ = self.get_texture(&asset);
            };
        }
    }
}

impl ApplicationHandler<Graphics> for Engine {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = self.last_frame.elapsed();

        if self.fps_specified {
            let target = self.last_frame + self.target_rate.unwrap_or_default();
            if now < target {
                event_loop.set_control_flow(ControlFlow::WaitUntil(target));
                return;
            }
        }

        self.last_frame = Instant::now();
        let _ = self.update(dt);
        let to_render = { self.to_render() };

        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };
        let _ = graphics.render(to_render);

        self.count += 1;
        if self.count > SAFETY_MAX_FOR_DEV {
            event_loop.exit()
        }

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_inner_size(LogicalSize::new(self.width, self.height));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let default_camera = Camera::new(
            self.width,
            self.height,
            crate::camera::camera::CameraMode::Orthographic2D,
        );
        self.graphics =
            Some(pollster::block_on(Graphics::new(window.clone(), default_camera)).unwrap());
        self.window = Some(window);

        self.setup();
        return;
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: Graphics) {
        self.graphics = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.width = size.width;
                self.height = size.height;
                let graphics = match &mut self.graphics {
                    Some(canvas) => canvas,
                    None => return,
                };
                graphics.resize(size.width, size.height)
            }
            WindowEvent::MouseInput {
                device_id: _,
                state: _,
                button: _,
            } => {
                let graphics = match &mut self.graphics {
                    Some(canvas) => canvas,
                    None => return,
                };
                graphics.set_background(Engine::random_color());
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                let game_state = self.to_render();
                let graphics = match &mut self.graphics {
                    Some(canvas) => canvas,
                    None => return,
                };
                // this is the only place we want to call graphics.render()
                // any other situation should use self.redraw();
                let _ = graphics.render(game_state);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => match (code, state.is_pressed()) {
                (KeyCode::Space, true) => {
                    self.redraw();
                }
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };
        graphics.process_camera_events(&event);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.cleanup()
    }
}

#[derive(Debug)]
struct LuaCameraKeyBinding {
    key: String,
    action: String,
}

#[derive(Debug)]
struct LuaCameraConfig {
    mode: String,
    speed: f32,
    keys: Vec<LuaCameraKeyBinding>,
}

impl LuaCameraConfig {
    fn from_lua_table(table: Table) -> Result<Self> {
        let mode: String = table.get("mode")?;
        let speed: f32 = table.get("speed")?;

        let keys_table: Table = table.get("keys")?;
        let mut keys = Vec::new();

        for pair in keys_table.sequence_values::<Table>() {
            let key_binding_table = pair?;
            let key: String = key_binding_table.get("key")?;
            let action: String = key_binding_table.get("action")?;

            keys.push(LuaCameraKeyBinding { key, action });
        }

        Ok(LuaCameraConfig { mode, speed, keys })
    }
    fn parse_keycode(s: &str) -> Result<KeyCode> {
        use KeyCode::*;
        let code = match s {
            "W" => KeyW,
            "A" => KeyA,
            "S" => KeyS,
            "D" => KeyD,
            "Q" => KeyQ,
            "E" => KeyE,
            "Up" => ArrowUp,
            "Down" => ArrowDown,
            "Left" => ArrowLeft,
            "Right" => ArrowRight,
            other => return Err(mlua::Error::RuntimeError(format!("Unknown key: {}", other))),
        };
        Ok(code)
    }

    fn parse_camera_action(s: &str) -> Result<CameraAction> {
        use CameraAction::*;
        let action = match s {
            "MoveForward" => MoveForward,
            "MoveBackward" => MoveBackward,
            "MoveLeft" => MoveLeft,
            "MoveRight" => MoveRight,
            "MoveUp" => MoveUp,
            "MoveDown" => MoveDown,
            "YawLeft" => YawLeft,
            "YawRight" => YawRight,
            "PitchUp" => PitchUp,
            "PitchDown" => PitchDown,
            "RollLeft" => RollLeft,
            "RollRight" => RollRight,
            other => {
                return Err(mlua::Error::RuntimeError(format!(
                    "Unknown camera action: {}",
                    other
                )))
            }
        };
        Ok(action)
    }
}
