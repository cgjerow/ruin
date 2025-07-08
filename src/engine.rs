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
    graphics: Option<Graphics>,
    count: u64,
    #[allow(dead_code)]
    fps_specified: bool,
    target_rate: Option<Duration>,
    last_frame: Instant,
    debugger: Debug,
    asset_cache: HashMap<String, Texture>,
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
            graphics: None,
            debugger: Debug::new(config.debug_enabled),
            count: 0,
            fps_specified: fps_opt != None,
            target_rate: target_rate,
            last_frame: Instant::now() - target_rate.unwrap_or_default(),
            asset_cache: HashMap::new(),
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

    fn update(&mut self) {
        debug_log!(self.debugger, "Updated it? {}", true)
    }

    fn cleanup(&mut self) {
        debug_log!(self.debugger, "Cleaned it? {}", true)
    }

    fn game_state(&mut self) -> GameState {
        self.generate_fake_game_state()
    }

    fn generate_fake_game_state(&mut self) -> GameState {
        let tree = self.get_texture("happy_tree.png");
        let mut rng = rand::rng();

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

        GameState {
            player,
            enemies,
            tree,
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
        self.update();
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
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.graphics = Some(pollster::block_on(Graphics::new(window)).unwrap());
        }
        self.get_texture("happy_tree.png");
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
        let game_state = self.game_state();
        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => graphics.resize(size.width, size.height),
            WindowEvent::MouseInput {
                device_id: _,
                state: _,
                button: _,
            } => {
                graphics.set_background(Engine::random_color());
                let _ = graphics.render(game_state);
            }
            WindowEvent::RedrawRequested => {
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
}
