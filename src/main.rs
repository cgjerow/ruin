use mlua::prelude::*;
use once_cell::sync::OnceCell;
use std::fs;
use std::sync::Arc;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

#[derive(Debug)]
struct Config {
    #[allow(dead_code)]
    fps: Duration,
    target_rate: Duration,
}

static CONFIG: OnceCell<Config> = OnceCell::new();
static SAFETY_MAX_FOR_DEV: u64 = 100;

struct Graphics {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    #[allow(dead_code)]
    window: Arc<Window>,
}

impl Graphics {
    async fn new(window: Arc<Window>) -> anyhow::Result<Graphics> {
        let size = window.inner_size();
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}

pub struct Engine {
    graphics: Option<Graphics>,
    last_frame: Instant,
    count: u64,
}

impl Engine {
    pub fn new() -> Self {
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

        Self {
            graphics: None,
            last_frame: Instant::now(),
            count: 0,
        }
    }

    fn update(&mut self) {
        println!("{}", CONFIG.get().unwrap().target_rate.as_millis());
        println!("{}", self.last_frame.elapsed().as_millis());
        sleep(
            CONFIG
                .get()
                .unwrap()
                .target_rate
                .saturating_sub(self.last_frame.elapsed()), // panics if negative, use sat_sub
        );
        println!(
            "update: time since last frame {}",
            self.last_frame.elapsed().as_millis()
        )
    }

    fn cleanup(&mut self) {
        println!("clean!")
    }
}

impl ApplicationHandler<Graphics> for Engine {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.update();

        let _ = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        }
        .render();

        self.last_frame = Instant::now();
        self.count += 1;
        if self.count > SAFETY_MAX_FOR_DEV {
            event_loop.exit()
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to await
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

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = Engine::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
