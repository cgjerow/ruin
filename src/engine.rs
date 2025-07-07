use crate::{debug, graphics};
use debug::Debug;
use graphics::Graphics;
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
        }
    }

    fn update(&mut self) {
        let _ = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        }
        .set_background(wgpu::Color {
            r: rand::random(),
            b: rand::random(),
            g: rand::random(),
            a: rand::random(),
        });
        debug_log!(self.debugger, "Updated it? {}", true)
    }

    fn cleanup(&mut self) {
        debug_log!(self.debugger, "Cleaned it? {}", true)
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

        let _ = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        }
        .render();

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
        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => graphics.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let _ = graphics.render();
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
