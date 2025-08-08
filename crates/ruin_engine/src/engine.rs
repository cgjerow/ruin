use cgmath::Vector2;
use mlua::{Result, Table};
use ruin_assets::{Handle, ImageTexture};
use ruin_bitmaps::vecbool_to_u8;
use ruin_camera::{Camera2D, Camera2DConfig, CameraOption, Dimensions};
use ruin_canvas::{parse_scene_from_lua, Canvas};
use ruin_debug::{debug_log, Debug};
use ruin_ecs::physics_2d::{Area2D, Body2D, BodyType2D, PhysicsWorld, Point2D, Shape2D, Vector2D};
use ruin_ecs::world::World;
use ruin_ecs::{
    animation_system_update_frames, set_entity_state, ActionState, ActionStateComponent, Animation,
    AnimationComponent, Entity, FlipComponent, HealthComponent, Transform2D,
};
use ruin_graphics::graphics_2d::Graphics2D;
use ruin_graphics::Graphics;
use ruin_lua_runtime::LuaExtendedExecutor;
use ruin_player_controller::{keycode_to_str, mousebutton_to_str};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

static SAFETY_MAX_FOR_DEV: u64 = 10000;

#[derive(Debug)]
struct FPS {
    pub frame_count: u32,
    pub time_accum: f32,
}

pub struct Engine {
    player: Entity,
    mouse_pos: [f32; 2], // TODO!!!
    //
    physics_tick_rate: f32,
    physics_accumulator: f32,
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
    lua_context: LuaExtendedExecutor,
    world: World,
    physics: PhysicsWorld,
    canvas: Canvas,
    width: u32,
    height: u32,
    fps: FPS,
    camera2d_config: Camera2DConfig,
}

pub struct EngineConfig {
    pub fps: String,
    pub debug_enabled: bool,
    pub width: u32,
    pub height: u32,
    pub dimensions: Dimensions,
    pub camera: CameraOption,
    pub camera2d_config: Camera2DConfig,
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
            mouse_pos: [0.0, 0.0],
            player: 0,
            physics_tick_rate: 1.0 / 60.0,
            physics_accumulator: 0.0,
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
            width: config.width,
            height: config.height,
            world: World::new(),
            physics: PhysicsWorld::new(),
            canvas: Canvas::new(),
            fps: FPS {
                frame_count: 0,
                time_accum: 0.0,
            },
            camera2d_config: config.camera2d_config,
        }
    }

    fn load_texture(&mut self, id: String) -> Handle<ImageTexture> {
        let path = format!("./assets/{}", id);
        self.graphics
            .as_mut()
            .expect("Graphics not initialized")
            .load_texture_from_path(&id, &path)
    }

    fn flip(&mut self, entity: u32, x: bool, y: bool) {
        self.world.flips.insert(entity, FlipComponent { x, y });
        if let Some(t) = self.world.transforms_2d.get_mut(&entity) {
            t.scale.x = if x { -t.scale.x.abs() } else { t.scale.x.abs() };
            t.scale.y = if y { -t.scale.y.abs() } else { t.scale.y.abs() };
        }
    }

    pub fn update_camera_follow_player(&mut self) {
        if self.dimensions == Dimensions::Two {
            if let Some(transform) = self.world.transforms_2d.get(&self.player) {
                let velocity = self.physics.get_velocity(&self.player);
                let graphics = match &mut self.graphics {
                    Some(canvas) => canvas,
                    None => return,
                };
                graphics.move_camera_for_follow(
                    [transform.position[0], transform.position[1], 0.0],
                    [velocity.x, velocity.y, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                );
            }
        }
    }

    fn is_targetting_fps(&self) -> bool {
        return self.fps_specified;
    }

    fn update(&mut self, dt: Duration) -> anyhow::Result<()> {
        let dt32 = dt.as_secs_f32();
        let update: mlua::Function = self.lua_context.get_function("ENGINE_update");

        self.physics_accumulator += dt32;

        let _ = update.call::<()>(dt32);

        while self.physics_accumulator >= self.physics_tick_rate {
            self.physics_accumulator -= self.physics_tick_rate;

            if self.dimensions == Dimensions::Two {
                self.physics.step(self.physics_tick_rate);
                if self.camera_mode == CameraOption::Follow {
                    self.update_camera_follow_player();
                }
            }
            self.world.update_positions(self.physics.positions());
        }

        let _ = self
            .lua_context
            .get_function("ENGINE_after_physics")
            .call::<()>(dt32);

        animation_system_update_frames(&mut self.world, dt32);
        return Ok(());
    }

    fn cleanup(&mut self) {
        debug_log!(self.debugger, "Cleaned it? {}", true)
    }

    fn get_window_size(&self) -> [u32; 2] {
        [self.width, self.height]
    }

    fn apply_force_2d(&mut self, id: Entity, fx: f32, fy: f32) {
        // TODO
    }

    fn apply_impulse_2d(&mut self, id: Entity, fx: f32, fy: f32) {
        // TODO
    }

    fn set_velocity_2d(&mut self, id: Entity, vx: f32, vy: f32) {
        self.physics.set_velocity(&id, Vector2D::new(vx, vy));
    }

    fn apply_masks_and_layers(&mut self, id: Entity, masks: Table, layers: Table) {
        let masks = vecbool_to_u8(LuaExtendedExecutor::table_to_vec_8(masks));
        let layers = vecbool_to_u8(LuaExtendedExecutor::table_to_vec_8(layers));

        self.world.update_area_masks_and_layers(&id, masks, layers);
    }

    fn toggle_area(&mut self, id: Entity, active: bool) {
        self.world.toggle_area(&id, active);
    }

    fn get_velocity_2d(&mut self, id: Entity) -> [f32; 2] {
        self.physics.get_velocity(&id).into()
    }

    fn get_position_2d(&mut self, id: Entity) -> [f32; 2] {
        if self.dimensions == Dimensions::Two {
            let position = self.world.transforms_2d.get(&id).unwrap().position;
            return [position.x, position.y];
        }
        [0.0, 0.0]
    }

    fn apply_move_2d(&mut self, id: Entity, x: f32, y: f32) {
        // TODO
        let t = self.world.transforms_2d.get_mut(&id).unwrap();
        t.position += Vector2::new(x, y);
    }

    fn damage(&mut self, id: Entity, amount: u16) -> bool {
        ruin_ecs::damage(&mut self.world, &id, amount)
    }

    fn get_health_table(&self, id: Entity) -> Table {
        let h = self
            .world
            .health_bars
            .get(&id)
            .unwrap_or(&HealthComponent {
                total: 0,
                current: 0,
            })
            .clone();
        let health = self.lua_context.create_table();
        let _ = health.set("total", h.total);
        let _ = health.set("current", h.current);
        health
    }

    fn set_state(&mut self, id: Entity, state: u8) {
        set_entity_state(&mut self.world, id, ActionState::from(state.clone()));
    }

    fn create_ui_scene(&mut self, lua_scene: mlua::Table) -> [u32; 1] {
        let entity = self.canvas.new_entity();
        let scene = parse_scene_from_lua(lua_scene, &mut self.canvas);
        for (_, element) in scene.0.elements.iter() {
            self.load_texture(element.sprite_sheet.clone());
        }
        self.canvas.add_scene(entity.clone(), scene);
        [entity.into()]
    }

    fn create_body(&mut self, lua_element: mlua::Table) -> [u32; 2] {
        let state: ActionState = lua_element.get("state").unwrap_or(0).into();
        let is_pc: bool = lua_element.get("is_pc").unwrap_or(false).into();
        let x: f32 = lua_element.get("x").unwrap_or(0.0);
        let y: f32 = lua_element.get("y").unwrap_or(0.0);
        let _z: f32 = lua_element.get("z").unwrap_or(0.0);
        let width: f32 = lua_element.get("width").unwrap_or(1.0);
        let height: f32 = lua_element.get("height").unwrap_or(1.0);
        let _depth: f32 = lua_element.get("depth").unwrap_or(1.0);
        let health: u16 = lua_element.get("total_health").unwrap_or(10);
        let collision_box: mlua::Table = lua_element
            .get("collision_box")
            .unwrap_or(self.lua_context.create_table());
        let collision_box_x_modifier: f32 = collision_box.get("size_modifier_x").unwrap_or(1.0);
        let collision_box_y_modifier: f32 = collision_box.get("size_modifier_y").unwrap_or(1.0);

        let masks = LuaExtendedExecutor::table_to_vec_8(
            lua_element
                .get("masks")
                .unwrap_or(self.lua_context.create_table()),
        );

        let layers = LuaExtendedExecutor::table_to_vec_8(
            lua_element
                .get("layers")
                .unwrap_or(self.lua_context.create_table()),
        );

        let animations: mlua::Table = lua_element
            .get("animations")
            .unwrap_or(self.lua_context.create_table());

        let entity: Entity = self.world.new_entity();
        if is_pc {
            self.player = entity.clone();
        }
        let mut animations_map = HashMap::new();

        for pair in animations.clone().pairs::<mlua::Value, mlua::Table>() {
            if let Ok((key, tbl)) = pair {
                let numeric_key =
                    key.as_u32()
                        .expect("Numeric key required for Action States") as u8;
                let animation =
                    Animation::from_lua_table(tbl, &mut |path: String| self.load_texture(path));
                let action_state = ActionState::from(numeric_key);
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
                Transform2D {
                    position: Vector2::new(x, y),
                    scale: Vector2::new(width, height),
                    shape: Shape2D::Rectangle {
                        // hard coding for now
                        half_extents: Vector2 { x: 0.5, y: 0.5 },
                    },
                    rotation_radians: 0.0,
                },
            );
            self.world.health_bars.insert(
                entity.clone(),
                HealthComponent {
                    total: health.clone(),
                    current: health.clone(),
                },
            );
            self.world
                .action_states
                .insert(entity.clone(), ActionStateComponent { state });

            self.physics.add_body(
                entity.clone(),
                Body2D::new(
                    Point2D { x, y },
                    Vector2D { x: 0.0, y: 0.0 },
                    BodyType2D::from(lua_element.get("type").unwrap_or(0)),
                    true,
                ),
            );
            self.physics.add_collider(
                &entity,
                Area2D {
                    shape: Shape2D::Rectangle {
                        half_extents: cgmath::Vector2 {
                            x: 0.5 * collision_box_x_modifier * width, // assuming all entities are using the same tile size (1 world unit) for now
                            y: 0.5 * collision_box_y_modifier * height,
                        },
                    },
                    offset: Vector2 {
                        x: collision_box.get("offset_x").unwrap_or(0.0),
                        y: collision_box.get("offset_y").unwrap_or(0.0),
                    },
                    masks: vecbool_to_u8(masks),
                    layers: vecbool_to_u8(layers),
                    active: true,
                },
            );
        }
        [entity.into(), 0]
    }

    pub fn screen_to_world(&self, loc: [f32; 2]) -> [f32; 2] {
        let size = self.get_window_size();
        let half_screen = [size[0] as f32 * 0.5, size[1] as f32 * 0.5];
        // Pixel offset from screen center
        let offset = [loc[0] - half_screen[0], loc[1] - half_screen[1]];

        let graphics = match &self.graphics {
            Some(canvas) => canvas,
            None => return [0.0, 0.0],
        };

        let camera = graphics.get_camera_info();
        // World units per pixel
        let units_per_pixel = (camera.zoom * 2.0) / size[1] as f32;

        // Scaled world offset
        let world_offset = [offset[0] * units_per_pixel, offset[1] * units_per_pixel];

        // Final world position is camera center + offset
        [
            camera.position[0] + world_offset[0],
            camera.position[1] - world_offset[1], // Y is flipped (screen Y-down vs world Y-up)
        ]
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

    fn setup(&mut self) {
        macro_rules! expose_fn {
            // Function with return type
            ($lua:expr, $ptr:expr, $table:expr, $name:ident, ($($arg:ident : $typ:ty),*) -> $ret:ty) => {{
                let func = $lua.create_function(move |_, ($($arg,)*): ($($typ,)*)| {
                    let engine = unsafe { &mut *$ptr };
                    Ok::<$ret, mlua::Error>(engine.$name($($arg),*))
                }).expect("Failed to create Lua function");
                $table.set(stringify!($name), func).expect("Failed to register Lua function");
            }};

            // Function with no return value (i.e. returns ())
            ($lua:expr, $ptr:expr, $table:expr, $name:ident, ($($arg:ident : $typ:ty),*)) => {{
                let func = $lua.create_function(move |_, ($($arg,)*): ($($typ,)*)| {
                    let engine = unsafe { &mut *$ptr };
                    engine.$name($($arg),*);
                    Ok(())
                }).expect("Failed to create Lua function");
                $table.set(stringify!($name), func).expect("Failed to register Lua function");
            }};
        }

        let self_ptr = self as *mut Self;
        let lua_engine = self.lua_context.create_table();
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, flip, (id: u32, x: bool, y: bool));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, apply_force_2d, (id: u32, x: f32, y: f32));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, apply_impulse_2d, (id: u32, x: f32, y: f32));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, apply_move_2d, (id: u32, x: f32, y: f32));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, apply_masks_and_layers, (id: u32, masks: Table, layers: Table));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, toggle_area, (id: u32, b: bool));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, set_velocity_2d, (id: u32, x: f32, y: f32));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, set_state, (id: u32, state: u8));
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, get_window_size, () -> [u32; 2]);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, get_velocity_2d, (id: u32) -> [f32; 2]);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, get_position_2d, (id: u32) -> [f32; 2]);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, damage, (id: u32, amount: u16) -> bool);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, get_health_table, (id: u32) -> Table);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, create_body, (data: Table) -> [u32; 2]);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, create_ui_scene, (data: Table) -> [u32; 1]);
        expose_fn!(self.lua_context.lua, self_ptr, lua_engine, configure_camera, (data: Table) -> Result<()>);

        let now_ns = self
            .lua_context
            .lua
            .create_function(|_, ()| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                Ok(now as u64) // or u128 if needed
            })
            .expect("Could not create function");
        lua_engine
            .set("now_ns", now_ns)
            .expect("Could not set engine function");

        self.lua_context
            .lua
            .globals()
            .set("engine", lua_engine)
            .expect("Could not define global engine");

        let config: mlua::Table = self
            .lua_context
            .get_function("ENGINE_load")
            .call::<mlua::Table>({})
            .expect("Unable to load initial assets.");

        let assets = config
            .get::<mlua::Table>("assets")
            .unwrap_or_else(|_| self.lua_context.create_table());
        for asset in assets.sequence_values::<String>() {
            let asset = asset.unwrap_or("".to_string());
            if asset != "" {
                self.load_texture(asset.clone());
            };
        }
    }

    fn call_lua_keyboard_input(&self, key: KeyCode, is_pressed: bool) {
        let _ = self
            .lua_context
            .get_function("ENGINE_input_event")
            .call::<()>((
                keycode_to_str(key),
                is_pressed,
                self.screen_to_world(self.mouse_pos),
            ));
    }

    fn call_lua_mouse_button_input(&self, button: MouseButton, is_pressed: bool) {
        let _ = self
            .lua_context
            .get_function("ENGINE_input_event")
            .call::<()>((
                mousebutton_to_str(button),
                is_pressed,
                self.screen_to_world(self.mouse_pos),
            ));
    }

    fn tick_game(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = self.last_frame.elapsed();

        self.fps.frame_count += 1;
        self.fps.time_accum += dt.as_secs_f32();
        if self.fps.time_accum > 1.0 {
            println!("FPS: {:?}", self.fps.frame_count);
            self.fps.time_accum = 0.0;
            self.fps.frame_count = 0;
        }

        if self.is_targetting_fps() {
            let target = self.last_frame + self.target_rate.unwrap_or_default();
            if now < target {
                event_loop.set_control_flow(ControlFlow::WaitUntil(target));
                return;
            }
        }

        self.last_frame = now;
        let bp = Instant::now();
        let _ = self.update(dt);

        //println!("Physics: {:?}", bp.elapsed().as_secs_f64());

        let graphics = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };
        let _ = graphics.update_camera();
        let bg = Instant::now();
        let _ = graphics.render(&self.world, &self.canvas, &self.physics);
        //println!("Render: {:?}", bg.elapsed().as_secs_f64());

        self.count += 1;
        if self.count > SAFETY_MAX_FOR_DEV {
            event_loop.exit()
        }

        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_inner_size(LogicalSize::new(self.width, self.height));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        if self.dimensions == Dimensions::Two {
            let camera_2d = Camera2D::new(&self.camera2d_config);
            self.graphics = Some(Box::new(
                pollster::block_on(Graphics2D::new(window.clone(), camera_2d)).unwrap(),
            ));
        }

        self.window = Some(window);

        self.setup();
    }
}

impl ApplicationHandler<Graphics2D> for Engine {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.tick_game(event_loop);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.initialize(event_loop);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.cleanup()
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
                state,
                button,
            } => {
                self.call_lua_mouse_button_input(button, state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Update mouse position
                self.mouse_pos = [position.x as f32, position.y as f32];
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
}
