use std::collections::HashMap;

use crate::{
    game_element::{
        ActionStateComponent, AnimationComponent, Entity, SpriteSheetComponent, TransformComponent,
    },
    graphics::{RenderElement, RenderQueue},
};

#[derive(Debug, Clone)]
pub struct World {
    next_id: u32,
    pub animations: HashMap<Entity, AnimationComponent>,
    pub sprite_sheets: HashMap<Entity, SpriteSheetComponent>,
    pub transforms: HashMap<Entity, TransformComponent>,
    pub action_states: HashMap<Entity, ActionStateComponent>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            transforms: HashMap::new(),
            action_states: HashMap::new(),
            animations: HashMap::new(),
            sprite_sheets: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        return entity;
    }

    pub fn extract_render_queue(&self) -> RenderQueue {
        let mut elements = Vec::new();

        for (entity, sprite) in &self.sprite_sheets {
            if let Some(transform_component) = self.transforms.get(entity) {
                let uv_coords = self
                    .animations
                    .get(entity)
                    .and_then(|anim| Some(anim.current_frame.uv_coords))
                    .unwrap_or([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]); // default full-texture UV

                elements.push(RenderElement {
                    position: transform_component.position,
                    size: transform_component.size,
                    texture: sprite.texture.clone(),
                    texture_id: sprite.texture_id.clone(),
                    uv_coords,
                });
            }
        }

        RenderQueue { elements }
    }
}
