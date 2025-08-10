use std::collections::{HashMap, HashSet};

use cgmath::{Array, InnerSpace, Vector2};
use ruin_assets::{Handle, ImageTexture};
use ruin_ecs::{
    physics_2d::{HalfExtents, Shape2D},
    world::{RenderElement2D, RenderQueue2D},
    ActionState, ActionStateComponent, Animation, AnimationComponent, Entity,
};
use ruin_lua_runtime::LuaExtendedExecutor;

pub type Index = u32;
pub type Resolution = Vector2<u32>;

#[derive(Debug)]
pub struct CanvasNode {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    animation: AnimationComponent,
    pub elements: Vec<CanvasNode>,
    active_elements: Vec<Index>,
}

#[derive(Debug)]
pub struct CanvasView {
    pub elements: Vec<CanvasNode>,
    active_elements: Vec<Index>,
}

#[derive(Debug)]
pub struct Canvas {
    next_id: u32,
    views: HashMap<Entity, CanvasView>,
    active_views: HashSet<Entity>,
    action_states: HashMap<Entity, ActionStateComponent>,
    virtual_resolution: Resolution,
}

impl Canvas {
    pub fn new(virtual_width: u32, virtual_height: u32) -> Self {
        Self {
            next_id: 0,
            views: HashMap::new(),
            active_views: HashSet::new(),
            action_states: HashMap::new(),
            virtual_resolution: Vector2 {
                x: virtual_width,
                y: virtual_height,
            },
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let e = self.next_id;
        self.next_id += 1;
        return e;
    }

    pub fn deactivate(&mut self, index: &Entity) {
        self.active_views.remove(index);
    }

    pub fn activate(&mut self, index: Entity) {
        self.active_views.insert(index);
    }

    pub fn add_view(&mut self, entity: Entity, scene: (CanvasView, bool)) {
        self.views.insert(entity.clone(), scene.0);
        if scene.1 {
            self.active_views.insert(entity.clone());
        }
    }

    pub fn extract_render_queue_2d(&self) -> RenderQueue2D {
        let mut queue = RenderQueue2D {
            transparent: Vec::new(),
            opaque: Vec::new(),
        };

        for entity in self.active_views.iter() {
            for element in self.views.get(entity).unwrap().elements.iter() {
                let shape = Shape2D::Rectangle {
                    half_extents: HalfExtents {
                        x: element.animation.current_frame.shape.half_extents().x
                            / (self.virtual_resolution.x as f32),
                        y: element.animation.current_frame.shape.half_extents().y
                            / (self.virtual_resolution.y as f32),
                    },
                };
                queue.transparent.push(RenderElement2D {
                    shape: shape,
                    position: element.position.into(),
                    size: element.scale.into(),
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
pub fn parse_canvas_view_from_lua(
    table: mlua::Table,
    texture_loader: &mut impl FnMut(String) -> Handle<ImageTexture>,
) -> (CanvasView, bool) {
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
        CanvasView {
            elements,
            active_elements,
        },
        table.get("initially_active").unwrap_or(false),
    )
}

fn parse_element_from_lua(
    table: mlua::Table,
    texture_loader: &mut impl FnMut(String) -> Handle<ImageTexture>,
) -> (CanvasNode, bool) {
    println!("{:?}", LuaExtendedExecutor::pretty_print_table(&table, 0));
    let first: mlua::Table = table.get("animations").unwrap();
    let x: f32 = table.get("tex_width").unwrap();
    let y: f32 = table.get("tex_height").unwrap();

    println!("{:?}", LuaExtendedExecutor::pretty_print_table(&first, 0));
    println!("{:?}", Vector2 { x, y });
    println!("{:?}", Vector2 { x, y }.normalize());
    let animation = Animation::raw_from_lua_table(first.get(0).unwrap(), texture_loader);
    let current_frame = animation.frames[0].clone();
    let animations = HashMap::from([(ActionState::Custom(0), animation)]);
    (
        CanvasNode {
            position: Vector2 {
                x: table.get("position_x").unwrap(),
                y: table.get("position_y").unwrap(),
            },
            scale: Vector2 {
                x: table.get("scale_x").unwrap(),
                y: table.get("scale_y").unwrap(),
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
