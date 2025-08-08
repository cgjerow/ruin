use std::collections::HashMap;

use cgmath::Vector2;
use ruin_assets::{Handle, ImageTexture};
use ruin_ecs::{
    physics_2d::Shape2D,
    world::{RenderElement2D, RenderQueue2D},
    ActionState, ActionStateComponent, Animation, AnimationComponent, Entity,
};
use ruin_lua_runtime::LuaExtendedExecutor;

pub type Index = u32;

#[derive(Debug)]
pub struct CanvasElement {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    shape: Shape2D,
    animation: AnimationComponent,
    pub elements: Vec<CanvasElement>,
    active_elements: Vec<Index>,
}

#[derive(Debug)]
pub struct CanvasScene {
    pub elements: Vec<CanvasElement>,
    active_elements: Vec<Index>,
}

#[derive(Debug)]
pub struct Canvas {
    next_id: u32,
    scenes: HashMap<Entity, CanvasScene>,
    active_scenes: Vec<Entity>,
    action_states: HashMap<Entity, ActionStateComponent>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            scenes: HashMap::new(),
            active_scenes: Vec::new(),
            action_states: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let e = self.next_id;
        self.next_id += 1;
        return e;
    }

    pub fn add_scene(&mut self, entity: Entity, scene: (CanvasScene, bool)) {
        self.scenes.insert(entity.clone(), scene.0);
        if scene.1 {
            self.active_scenes.push(entity.clone());
        }
    }

    pub fn extract_render_queue_2d(&self) -> RenderQueue2D {
        let mut queue = RenderQueue2D {
            transparent: Vec::new(),
            opaque: Vec::new(),
        };

        for entity in self.active_scenes.iter() {
            for element in self.scenes.get(entity).unwrap().elements.iter() {
                queue.transparent.push(RenderElement2D {
                    shape: &Shape2D::Rectangle {
                        half_extents: Vector2 { x: 1.0, y: 1.0 },
                    },
                    position: [0.0, 0.0],
                    size: [1.0, 1.0],
                    z_order: 0.0,
                    image_texture: element
                        .animation
                        .animations
                        .get(&ActionState::Custom(0))
                        .unwrap()
                        .sprite_sheet_id,
                    uv_coords: element.animation.current_frame.uv_coords,
                });
            }
        }

        queue
    }
}

// Lua
pub fn parse_scene_from_lua(
    table: mlua::Table,
    texture_loader: &mut impl FnMut(String) -> Handle<ImageTexture>,
) -> (CanvasScene, bool) {
    let elements_table: mlua::Table = table.get("elements").unwrap();

    let mut elements = Vec::new();
    let mut active_elements = Vec::<Entity>::new();

    let mut index = 0;
    for val in elements_table.sequence_values() {
        let e_tup = parse_element_from_lua(val.unwrap(), texture_loader);
        if e_tup.1 {
            active_elements.push(index);
        }
        elements.push(e_tup.0);
        index += 1;
    }

    (
        CanvasScene {
            elements,
            active_elements,
        },
        table.get("initially_active").unwrap_or(false),
    )
}

fn parse_element_from_lua(
    table: mlua::Table,
    texture_loader: &mut impl FnMut(String) -> Handle<ImageTexture>,
) -> (CanvasElement, bool) {
    println!("{:?}", LuaExtendedExecutor::pretty_print_table(&table, 0));
    let first: mlua::Table = table.get("animations").unwrap();
    println!("{:?}", LuaExtendedExecutor::pretty_print_table(&first, 0));
    let animation = Animation::from_lua_table(first.get(0).unwrap(), texture_loader);
    let current_frame = animation.frames[0].clone();
    let animations = HashMap::from([(ActionState::Custom(0), animation)]);
    (
        CanvasElement {
            position: Vector2 {
                x: table.get("position_x").unwrap(),
                y: table.get("position_y").unwrap(),
            },
            scale: Vector2 {
                x: table.get("scale_x").unwrap(),
                y: table.get("scale_y").unwrap(),
            },
            shape: Shape2D::Rectangle {
                half_extents: Vector2 {
                    x: table.get("width").unwrap(),
                    y: table.get("height").unwrap(),
                },
            },
            animation: AnimationComponent {
                animations,
                current_frame,
                current_frame_index: 1,
                frame_timer: 0.0,
            },
            elements: Vec::new(),
            active_elements: Vec::new(),
        },
        table.get("initially_active").unwrap_or(false),
    )
}
