use crate::camera::{Camera, CameraAction, CameraInputMap, TwoDimensionalCameraController};
use crate::game_element::{Animation, StatefulElement, VisualState};
use crate::lua_scriptor::LuaExtendedExecutor;
use crate::texture::Texture;
use crate::{debug, graphics};
use debug::Debug;
use graphics::Graphics;
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
            println!("Initialized asset: {}", path);

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

        println!("Created {}", id);
        self.character_cache.insert(id, character);
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

        let lua_engine = self.lua_context.create_table();
        lua_engine
            .set("get_window_size", get_window_size)
            .expect("Could not set engine function");
        lua_engine
            .set("create_character", create_character)
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
        let camera = Camera::new(
            self.width,
            self.height,
            crate::camera::camera::CameraMode::Orthographic2D,
        );

        let input_map = CameraInputMap::new()
            .insert(KeyCode::KeyW, CameraAction::MoveForward)
            .insert(KeyCode::KeyS, CameraAction::MoveBackward)
            .insert(KeyCode::KeyA, CameraAction::MoveLeft)
            .insert(KeyCode::KeyD, CameraAction::MoveRight)
            .insert(KeyCode::KeyQ, CameraAction::RollLeft)
            .insert(KeyCode::KeyE, CameraAction::RollRight)
            .insert(KeyCode::ArrowUp, CameraAction::PitchUp)
            .insert(KeyCode::ArrowDown, CameraAction::PitchDown)
            .insert(KeyCode::ArrowLeft, CameraAction::YawLeft)
            .insert(KeyCode::ArrowRight, CameraAction::YawRight);

        //let camera_controller = Box::new(UniversalCameraController::new(10.0, 5.0, input_map));
        let camera_controller = Box::new(TwoDimensionalCameraController::new(10.0));
        self.graphics = Some(
            pollster::block_on(Graphics::new(window.clone(), camera, camera_controller)).unwrap(),
        );
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
        graphics.camera_controller.process_events(&event);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.cleanup()
    }
}

pub struct ElementsToRender {
    pub elements: Vec<StatefulElement>,
}
