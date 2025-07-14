use std::collections::HashMap;

use crate::{
    game_element::{
        ActionStateComponent, AnimationComponent, Entity, FlipComponent, SpriteSheetComponent,
        TransformComponent,
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
    pub flips: HashMap<Entity, FlipComponent>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            transforms: HashMap::new(),
            action_states: HashMap::new(),
            animations: HashMap::new(),
            sprite_sheets: HashMap::new(),
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
            if let Some(transform_component) = self.transforms.get(entity) {
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
                    .unwrap();
                elements.push(RenderElement {
                    position: transform_component.position,
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
}
