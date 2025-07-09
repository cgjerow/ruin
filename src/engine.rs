use crate::lua_scriptor::LuaExtendedExecutor;
use crate::texture::Texture;
use crate::{debug, graphics};
use debug::Debug;
use graphics::Graphics;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
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
    game_state: Option<GameState>,
    lua_context: Arc<LuaExtendedExecutor>,
}

pub struct EngineConfig {
    pub fps: String,
    pub debug_enabled: bool,
}

impl Engine {
    pub fn run_event_loop(config: EngineConfig) -> anyhow::Result<()> {
        let event_loop = EventLoop::with_user_event().build()?;
        let mut app = Engine::new(config);
        event_loop.run_app(&mut app)?;
        return Ok(());
    }

    pub fn new(config: EngineConfig) -> Self {
        let fps_opt = if config.fps.trim().eq_ignore_ascii_case("auto") {
            None
        } else {
            config.fps.parse::<u64>().ok()
        };

        let target_rate = fps_opt.map(|fps| Duration::from_millis(1000 / fps));

        Self {
            lua_context: (Arc::new(LuaExtendedExecutor::new("main"))),
            window: None,
            graphics: None,
            debugger: Debug::new(config.debug_enabled),
            count: 0,
            fps_specified: fps_opt != None,
            target_rate: target_rate,
            last_frame: Instant::now() - target_rate.unwrap_or_default(),
            asset_cache: HashMap::new(),
            game_state: None,
        }
    }

    fn get_texture(&mut self, path: &str) -> Texture {
        let texture = self.asset_cache.entry(path.to_string()).or_insert_with(|| {
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

    fn update(&mut self) -> anyhow::Result<()> {
        debug_log!(self.debugger, "Updated it? {}", true);
        let update: mlua::Function = self.lua_context.get_function("update");
        let _ = update.call::<()>(1);
        return Ok(());
    }

    fn cleanup(&mut self) {
        debug_log!(self.debugger, "Cleaned it? {}", true)
    }

    fn game_state(&mut self) -> GameState {
        let tree = self.get_texture("happy_tree.png");
        let mittens = self.get_texture("mittens-goblin-art.png");
        let previous_game_state = self.game_state.take();
        let mut rng = rand::rng();
        let show_mittens = previous_game_state
            .as_ref()
            .map(|s| s.show_mittens)
            .unwrap_or(true); // default to true if no previous state

        // Player always starts at (1, 1)
        let player = Player {
            location: Position { x: 1.0, y: 1.0 },
        };

        // Generate 5 enemies with random locations
        let enemies = (0..5)
            .map(|i| Enemy {
                id: format!("enemy_{}", i),
                location: Position {
                    x: rng.random_range(0.0..10.0),
                    y: rng.random_range(0.0..10.0),
                },
            })
            .collect();

        self.game_state = Some(GameState {
            player,
            enemies,
            tree,
            mittens,
            show_mittens,
        });

        return GameState {
            player: Player {
                location: Position { x: 1.0, y: 1.0 },
            },
            enemies: (0..5)
                .map(|i| Enemy {
                    id: format!("enemy_{}", i),
                    location: Position {
                        x: rng.random_range(0.0..10.0),
                        y: rng.random_range(0.0..10.0),
                    },
                })
                .collect(),
            tree: self.get_texture("happy_tree.png"),
            mittens: self.get_texture("mittens-goblin-art.png"),
            show_mittens,
        };
    }

    fn flip_mittens(&mut self) {
        if let Some(state) = self.game_state.as_mut() {
            state.show_mittens = !state.show_mittens;
        }
    }

    fn load_initial_assets(&mut self) {
        let result_table: mlua::Table = self
            .lua_context
            .get_function("load")
            .call::<mlua::Table>({})
            .expect("Unable to load initial assets.");

        let assets = result_table
            .get::<mlua::Table>("assets")
            .unwrap_or_else(|_| self.lua_context.create_table());
        for asset in assets.sequence_values::<String>() {
            let asset = asset.unwrap_or("".to_string());
            if asset != "" {
                println!("Initialized asset: {}", asset);
                let _ = self.get_texture(&asset);
            };
        }
    }
}

impl ApplicationHandler<Graphics> for Engine {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        if self.fps_specified {
            let target = self.last_frame + self.target_rate.unwrap_or_default();
            if now < target {
                event_loop.set_control_flow(ControlFlow::WaitUntil(target));
                return;
            }
        }

        self.last_frame = Instant::now();
        let _ = self.update();
        let game_state = self.game_state();

        let _ = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        }
        .render(game_state);

        self.count += 1;
        if self.count > SAFETY_MAX_FOR_DEV {
            event_loop.exit()
        }

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.graphics = Some(pollster::block_on(Graphics::new(window.clone())).unwrap());
        self.window = Some(window);

        self.load_initial_assets();
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
                let game_state = self.game_state();
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
                    self.flip_mittens();
                    self.redraw();
                }
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.cleanup()
    }
}

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Player {
    pub location: Position,
}

#[derive(Debug)]
pub struct Enemy {
    pub id: String,
    pub location: Position,
}

#[derive(Debug)]
pub struct GameState {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub tree: Texture,
    pub mittens: Texture,
    pub show_mittens: bool,
}
