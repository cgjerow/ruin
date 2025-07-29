use std::collections::HashMap;

use cgmath::Vector2;

use crate::components_systems::{physics_2d::Shape2D, Entity};

#[derive(Debug)]
pub struct CanvasElement {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    shape: Shape2D,
}

#[derive(Debug)]
pub struct CanvasScene {
    scenes: HashMap<Entity, CanvasScene>,
    elements: HashMap<Entity, CanvasElement>,
    active_scenes: Vec<Entity>,
    active_elements: Vec<Entity>,
}

#[derive(Debug)]
pub struct Canvas {
    next_id: u32,
    scenes: HashMap<Entity, CanvasScene>,
    active_scenes: Vec<Entity>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            scenes: HashMap::new(),
            active_scenes: Vec::new(),
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
}

// Lua
pub fn parse_scene_from_lua(table: mlua::Table, canvas: &mut Canvas) -> (CanvasScene, bool) {
    let elements_table: mlua::Table = table.get("elements").unwrap();
    let scenes_table: mlua::Table = table.get("scenes").unwrap();

    let mut scenes = HashMap::new();
    let mut elements = HashMap::new();
    let mut active_elements = Vec::<Entity>::new();
    let mut active_scenes = Vec::<Entity>::new();

    for val in elements_table.sequence_values() {
        let id = canvas.new_entity();
        let e_tup = parse_element_from_lua(val.unwrap());
        if e_tup.1 {
            active_elements.push(id.clone());
        }
        elements.insert(id, e_tup.0);
    }

    for val in scenes_table.sequence_values() {
        let id = canvas.new_entity();
        let s_tup = parse_scene_from_lua(val.unwrap(), canvas);
        if s_tup.1 {
            active_scenes.push(id.clone());
        }
        scenes.insert(id, s_tup.0);
    }

    (
        CanvasScene {
            scenes,
            elements,
            active_scenes,
            active_elements,
        },
        table.get("active").unwrap_or(false),
    )
}

fn parse_element_from_lua(table: mlua::Table) -> (CanvasElement, bool) {
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
        },
        table.get("initially_active").unwrap_or(false),
    )
}
