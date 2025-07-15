use crate::camera_2d::Camera2D;
use crate::camera_3d::{Camera3D, CameraAction};
use crate::components_systems::physics_2d::{
    self, ColliderComponent, FlipComponent, TransformComponent,
};
use crate::components_systems::{
    animation_system_update_frames, set_entity_state, ActionState, ActionStateComponent, Animation,
    AnimationComponent, Entity, SpriteSheetComponent,
};
use crate::graphics::Graphics;
use crate::lua_scriptor::LuaExtendedExecutor;
use crate::texture::Texture;
use crate::world::World;
use crate::{debug, graphics_2d, graphics_3d};
use debug::Debug;
use graphics_2d::Graphics2D;
use graphics_3d::Graphics3D;
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
    player: Entity,
    window: Option<Arc<Window>>,
    graphics: Option<Box<dyn Graphics>>,
    camera_mode: CameraOption,
    dimensions: Dimensions,
    count: u64,
    #[allow(dead_code)]
    fps_specified: bool,
    target_rate: Option<Duration>,
    last_frame: Instant,
    debugger: Debug,
    asset_cache: HashMap<String, Texture>,
    lua_context: LuaExtendedExecutor,
    world: World,
    width: u32,
    height: u32,
}

pub struct EngineConfig {
    pub fps: String,
    pub debug_enabled: bool,
    pub width: u32,
    pub height: u32,
    pub dimensions: Dimensions,
    pub camera: CameraOption,
}

#[derive(Debug, PartialEq)]
pub enum Dimensions {
    Two,
    Three,
}

#[derive(Debug, PartialEq)]
pub enum CameraOption {
    Follow,
    Independent,
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
            player: Entity(0),
            lua_context: lua_executor,
            window: None,
            graphics: None,
            camera_mode: config.camera,
            dimensions: config.dimensions,
            debugger: Debug::new(config.debug_enabled),
            count: 0,
            fps_specified: fps_opt != None,
            target_rate: target_rate,
            last_frame: Instant::now() - target_rate.unwrap_or_default(),
            asset_cache: HashMap::new(),
            width: config.width,
            height: config.height,
            world: World::new(),
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

    fn flip(&mut self, entity: u32, x: bool, y: bool) {
        self.world
            .flips
            .insert(Entity(entity), FlipComponent { x, y });
    }

    pub fn update_camera_follow(&mut self, entity: &Entity) {
        if self.dimensions == Dimensions::Two {
            if let Some(transform) = self.world.transforms_2d.get(entity) {
                let graphics = match &mut self.graphics {
                    Some(canvas) => canvas,
                    None => return,
                };
                graphics.move_camera_for_follow(
                    [transform.position[0], transform.position[1], 0.0],
                    [transform.velocity[0], transform.velocity[1], 0.0],
                    [transform.acceleration[0], transform.acceleration[1], 0.0],
                    [0.0, 0.0, 0.0],
                );
            }
        }
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
        let update: mlua::Function = self.lua_context.get_function("update");
        let dt32 = dt.as_secs_f32();

        let _ = update.call::<()>(dt32);

        if self.dimensions == Dimensions::Two {
            let next_transforms =
                physics_2d::transform_system_calculate_intended_position(&self.world, dt32);
            let collisions = physics_2d::collision_system(&self.world, &next_transforms);
            let collisions_table = self
                .lua_context
                .rust_collisions_to_lua_2d(collisions)
                .unwrap();

            self.lua_context
                .get_function("on_collision")
                .call::<()>(collisions_table)
                .expect("Error handling collisions");

            let transform_notices = physics_2d::transform_system_physics(&mut self.world, dt32);
            self.on_entity_idle(transform_notices.idled);

            if self.camera_mode == CameraOption::Follow {
                self.update_camera_follow(&self.player.clone());
            }
        }

        animation_system_update_frames(&mut self.world, dt32);

        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return Ok(()),
        };

        graphics.update_camera();

        return Ok(());
    }

    fn on_entity_idle(&self, entities: Vec<Entity>) {
        let on_idle: mlua::Function = self.lua_context.get_function("on_entity_idle");
        let entity_ids: Vec<u32> = entities.iter().map(|entity| entity.0).collect();
        on_idle
            .call::<()>(entity_ids)
            .expect("Error calling on_entity_idle")
    }

    fn cleanup(&mut self) {
        debug_log!(self.debugger, "Cleaned it? {}", true)
    }

    fn get_window_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn add_acceleration(&mut self, id: u32, dx: f32, dy: f32) {
        if self.dimensions == Dimensions::Two {
            physics_2d::transform_system_add_acceleration(&mut self.world, Entity(id), dx, dy);
        }
    }

    fn redirect(
        &mut self,
        id: u32,
        dx: f32,
        dy: f32,
        sep_x: f32,
        sep_y: f32,
        acceleration_mod: f32,
    ) {
        if self.dimensions == Dimensions::Two {
            physics_2d::transform_system_redirect(
                &mut self.world,
                Entity(id),
                dx,
                dy,
                sep_x,
                sep_y,
                acceleration_mod,
            );
        }
    }

    fn set_state(&mut self, id: u32, state: String) {
        set_entity_state(
            &mut self.world,
            Entity(id),
            ActionState::from(state.clone()),
        );
    }

    fn create_character(&mut self, character_table: mlua::Table) -> u32 {
        let state: ActionState = character_table
            .get("state")
            .unwrap_or("Idle".to_string())
            .into();
        let is_pc: bool = character_table.get("is_pc").unwrap_or(false).into();
        let x: f32 = character_table.get("x").unwrap_or(0.0);
        let y: f32 = character_table.get("y").unwrap_or(0.0);
        let z: f32 = character_table.get("z").unwrap_or(0.0);
        let width: f32 = character_table.get("width").unwrap_or(1.0);
        let height: f32 = character_table.get("height").unwrap_or(1.0);

        let animations: mlua::Table = character_table
            .get("animations")
            .unwrap_or(self.lua_context.create_table());

        let entity: Entity = self.world.new_entity();
        if is_pc {
            self.player = entity.clone();
        }
        let mut animations_map = HashMap::new();

        for pair in animations.clone().pairs::<mlua::Value, mlua::Table>() {
            if let Ok((key, tbl)) = pair {
                let (mut animation, sprite_path) = Animation::from_lua_table(tbl);
                let action_state = ActionState::from(
                    key.to_string()
                        .expect("String key required for Action States"),
                );

                let sprite_id: Entity = self.world.new_entity();
                let texture = self.get_texture(&sprite_path);
                animation.sprite_sheet_id = sprite_id;

                self.world.sprite_sheets.insert(
                    sprite_id.clone(),
                    SpriteSheetComponent {
                        texture_id: sprite_path,
                        texture,
                    },
                );
                animations_map.insert(action_state, animation);
            }
        }

        let current_frame = animations_map.get(&state).unwrap().frames[0].clone();

        if self.dimensions == Dimensions::Two {
            self.world.animations.insert(
                entity.clone(),
                AnimationComponent {
                    animations: animations_map,
                    current_frame_index: 0,
                    current_frame,
                    frame_timer: 0.0,
                },
            );
            self.world.transforms_2d.insert(
                entity.clone(),
                TransformComponent {
                    position: [x, y],
                    velocity: [0.0, 0.0],
                    acceleration: [0.0, 0.0],
                    size: [width, height],
                },
            );
            self.world.colliders_2d.insert(
                entity.clone(),
                ColliderComponent {
                    offset: [0.0, 0.0],
                    size: [width * 0.6, height * 0.6],
                    is_solid: true,
                },
            );
            self.world
                .action_states
                .insert(entity.clone(), ActionStateComponent { state });
        }
        entity.into()
    }

    pub fn configure_camera(&mut self, _config: mlua::Table) -> Result<()> {
        // let config = LuaCameraConfig::from_lua_table(config)?;
        // Create camera
        // let camera = Camera3D::new(self.width, self.height, mode.clone());
        //
        /*
        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return Ok(()),
        };
        */
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
        let add_acceleration = self
            .lua_context
            .lua
            .create_function(move |_, (id, dx, dy): (u32, f32, f32)| {
                let engine = unsafe { &mut *self_ptr };
                Ok(engine.add_acceleration(id, dx, dy))
            })
            .expect("Could not create function");
        let redirect = self
            .lua_context
            .lua
            .create_function(
                move |_, (id, dx, dy, sep_x, sep_y, a): (u32, f32, f32, f32, f32, f32)| {
                    let engine = unsafe { &mut *self_ptr };
                    Ok(engine.redirect(id, dx, dy, sep_x, sep_y, a))
                },
            )
            .expect("Could not create function");
        let set_state = self
            .lua_context
            .lua
            .create_function(move |_, (id, state): (u32, String)| {
                let engine = unsafe { &mut *self_ptr };
                Ok(engine.set_state(id, state))
            })
            .expect("Could not create function");
        let flip = self
            .lua_context
            .lua
            .create_function(move |_, (id, x, y): (u32, bool, bool)| {
                let engine = unsafe { &mut *self_ptr };
                Ok(engine.flip(id, x, y))
            })
            .expect("Could not create function");

        let lua_engine = self.lua_context.create_table();
        lua_engine
            .set("get_window_size", get_window_size)
            .expect("Could not set engine function");
        lua_engine
            .set("flip", flip)
            .expect("Could not set engine function");
        lua_engine
            .set("add_acceleration", add_acceleration)
            .expect("Could not set engine function");
        lua_engine
            .set("redirect", redirect)
            .expect("Could not set engine function");
        lua_engine
            .set("set_state", set_state)
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

    fn call_lua_keyboard_input(&self, key: KeyCode, is_pressed: bool) {
        let _ = self
            .lua_context
            .get_function("keyboard_event")
            .call::<()>((keycode_to_str(key), is_pressed));
    }
}

impl ApplicationHandler<Graphics3D> for Engine {
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

        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };
        let _ = graphics.render(&self.world);

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

        if self.dimensions == Dimensions::Three {
            let camera_3d = Camera3D::new(
                self.width,
                self.height,
                crate::camera_3d::camera_3d::CameraMode::Orthographic2D,
            );
            self.graphics = Some(Box::new(
                pollster::block_on(Graphics3D::new(window.clone(), camera_3d)).unwrap(),
            ));
        } else if self.dimensions == Dimensions::Two {
            let camera_2d = Camera2D::new(self.width, self.height);
            self.graphics = Some(Box::new(
                pollster::block_on(Graphics2D::new(window.clone(), camera_2d)).unwrap(),
            ));
        }

        self.window = Some(window);

        self.setup();
        return;
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
                graphics.set_background(random_color());
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                let graphics = match &mut self.graphics {
                    Some(canvas) => canvas,
                    None => return,
                };
                // this is the only place we want to call graphics.render()
                // any other situation should use self.redraw();
                let _ = graphics.render(&self.world);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                match (code, state.is_pressed()) {
                    (KeyCode::Space, true) => {
                        self.redraw();
                    }
                    (KeyCode::Escape, true) => event_loop.exit(),
                    _ => {}
                };
                self.call_lua_keyboard_input(code, state.is_pressed());
            }
            _ => {}
        }

        if self.camera_mode == CameraOption::Independent {
            let graphics = match &mut self.graphics {
                Some(canvas) => canvas,
                None => return,
            };
            graphics.process_camera_event(&event);
        }
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
    locked: bool,
    keys: Vec<LuaCameraKeyBinding>,
}

impl LuaCameraConfig {
    fn from_lua_table(table: Table) -> Result<Self> {
        let mode: String = table.get("mode")?;
        let speed: f32 = table.get("speed")?;
        let locked: bool = table.get("locked")?;

        let keys_table: Table = table.get("keys")?;
        let mut keys = Vec::new();

        for pair in keys_table.sequence_values::<Table>() {
            let key_binding_table = pair?;
            let key: String = key_binding_table.get("key")?;
            let action: String = key_binding_table.get("action")?;

            keys.push(LuaCameraKeyBinding { key, action });
        }

        Ok(LuaCameraConfig {
            mode,
            speed,
            locked,
            keys,
        })
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

fn keycode_to_str(key: KeyCode) -> Option<&'static str> {
    use winit::keyboard::KeyCode::*;
    Some(match key {
        KeyW => "w",
        KeyA => "a",
        KeyS => "s",
        KeyD => "d",
        ArrowUp => "up",
        ArrowDown => "down",
        ArrowLeft => "left",
        ArrowRight => "right",
        Space => "space",
        Enter => "enter",
        Escape => "escape",
        KeyZ => "z",
        KeyX => "x",
        KeyC => "c",
        KeyV => "v",
        Digit0 => "0",
        Digit1 => "1",
        Digit2 => "2",
        Digit3 => "3",
        Digit4 => "4",
        Digit5 => "5",
        Digit6 => "6",
        Digit7 => "7",
        Digit8 => "8",
        Digit9 => "9",
        KeyQ => "q",
        KeyE => "e",
        KeyR => "r",
        KeyF => "f",
        KeyT => "t",
        KeyY => "y",
        KeyU => "u",
        KeyI => "i",
        KeyO => "o",
        KeyP => "p",
        KeyB => "b",
        KeyN => "n",
        KeyM => "m",
        _ => return None, // Unknown or unhandled key
    })
}

fn random_color() -> wgpu::Color {
    return wgpu::Color {
        r: rand::random(),
        b: rand::random(),
        g: rand::random(),
        a: rand::random(),
    };
}
