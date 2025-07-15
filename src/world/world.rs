use std::collections::HashMap;

use crate::{
    components_systems::{
        physics_2d::{ColliderComponent, FlipComponent, TransformComponent},
        physics_3d, ActionStateComponent, AnimationComponent, Entity, SpriteSheetComponent,
    },
    graphics_2d::{RenderElement2D, RenderQueue2D},
    graphics_3d::{RenderElement, RenderQueue},
};

#[derive(Debug, Clone)]
pub struct World {
    next_id: u32,
    pub animations: HashMap<Entity, AnimationComponent>,
    pub sprite_sheets: HashMap<Entity, SpriteSheetComponent>,
    pub transforms_2d: HashMap<Entity, TransformComponent>,
    pub transforms_3d: HashMap<Entity, physics_3d::TransformComponent>,
    pub action_states: HashMap<Entity, ActionStateComponent>,
    pub colliders_2d: HashMap<Entity, ColliderComponent>,
    pub colliders_3d: HashMap<Entity, physics_3d::ColliderComponent>,
    pub flips: HashMap<Entity, FlipComponent>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            transforms_2d: HashMap::new(),
            transforms_3d: HashMap::new(),
            action_states: HashMap::new(),
            animations: HashMap::new(),
            sprite_sheets: HashMap::new(),
            colliders_2d: HashMap::new(),
            colliders_3d: HashMap::new(),
            flips: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        return entity;
    }

    pub fn extract_render_queue(&self) -> RenderQueue {
        let mut elements = Vec::new();
        for (entity, animation) in &self.animations {
            if let Some(transform_component) = self.transforms_3d.get(entity) {
                let uv_coords = animation.current_frame.uv_coords;
                let flip = self
                    .flips
                    .get(entity)
                    .unwrap_or(&FlipComponent { x: false, y: false });

                let sprite = self
                    .sprite_sheets
                    .get(
                        &animation.animations[&self
                            .action_states
                            .get(&entity)
                            .expect("Animation not found")
                            .state]
                            .sprite_sheet_id,
                    )
                    .expect("Sprite Sheets not found");
                elements.push(RenderElement {
                    position: [
                        transform_component.position[0],
                        transform_component.position[1],
                        0.0,
                    ],
                    size: transform_component.size,
                    texture: sprite.texture.clone(),
                    texture_id: sprite.texture_id.clone(),
                    flip_x: flip.x,
                    flip_y: flip.y,
                    uv_coords,
                });
            }
        }

        RenderQueue { elements }
    }

    pub fn extract_render_queue_2d(&self) -> RenderQueue2D {
        let mut elements = Vec::new();

        for (entity, animation) in self.animations.iter() {
            if let Some(transform) = self.transforms_2d.get(entity) {
                let uv_coords = animation.current_frame.uv_coords;
                let flip = self
                    .flips
                    .get(entity)
                    .unwrap_or(&FlipComponent { x: false, y: false });
                let sprite = self
                    .sprite_sheets
                    .get(
                        &animation.animations[&self
                            .action_states
                            .get(&entity)
                            .expect("Animation not found")
                            .state]
                            .sprite_sheet_id,
                    )
                    .expect("Sprite Sheets not found");
                elements.push(RenderElement2D {
                    position: [transform.position[0], transform.position[1]],
                    size: transform.size,
                    z_order: -transform.position[1], // Sort top to bottom: lower y = drawn later
                    texture: sprite.texture.clone(),
                    texture_id: sprite.texture_id.clone(),
                    uv_coords,
                    flip_x: flip.x,
                    flip_y: flip.y,
                });
            }
        }

        RenderQueue2D { elements }
    }
}
