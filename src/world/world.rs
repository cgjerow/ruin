use std::{collections::HashMap, hash::Hash};

use cgmath::Vector2;

use crate::{
    components_systems::{
        physics_2d::{Area2D, ColliderComponent, FlipComponent, PhysicsBody, TransformComponent},
        physics_3d, ActionStateComponent, AnimationComponent, Entity, HealthComponent,
        SpriteSheetComponent,
    },
    graphics_2d::{RenderElement2D, RenderQueue2D},
    graphics_3d::{RenderElement, RenderQueue},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaRole {
    Physics,
    Hitbox,
    Trigger,
}

#[derive(Debug, Clone, Copy)]
pub struct AreaInfo {
    pub role: AreaRole,
    pub parent: Entity,
}

#[derive(Debug, Clone)]
pub struct World {
    next_id: u32,
    pub health_bars: HashMap<Entity, HealthComponent>,
    pub animations: HashMap<Entity, AnimationComponent>,
    pub sprite_sheets: HashMap<Entity, SpriteSheetComponent>,
    pub transforms_2d: HashMap<Entity, TransformComponent>,
    pub transforms_3d: HashMap<Entity, physics_3d::TransformComponent>,
    pub action_states: HashMap<Entity, ActionStateComponent>,
    pub physics_bodies_2d: HashMap<Entity, PhysicsBody>,
    pub physical_colliders_2d: HashMap<Entity, HashMap<Entity, Area2D>>,
    pub area_roles: HashMap<Entity, AreaInfo>,
    pub colliders_3d: HashMap<Entity, physics_3d::ColliderComponent>,
    pub flips: HashMap<Entity, FlipComponent>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            health_bars: HashMap::new(),
            transforms_2d: HashMap::new(),
            transforms_3d: HashMap::new(),
            action_states: HashMap::new(),
            animations: HashMap::new(),
            physics_bodies_2d: HashMap::new(),
            sprite_sheets: HashMap::new(),
            physical_colliders_2d: HashMap::new(),
            area_roles: HashMap::new(),
            colliders_3d: HashMap::new(),
            flips: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let entity = Entity(self.next_id);
        self.next_id += 1;
        return entity;
    }

    pub fn insert_area_2d(&mut self, info: AreaInfo, area: Area2D) -> Entity {
        let area_entity = self.new_entity();
        match info.role {
            AreaRole::Physics => {
                self.physical_colliders_2d
                    .entry(info.parent)
                    .or_insert_with(HashMap::new)
                    .insert(area_entity, area);
            }
            _ => {}
        }
        self.area_roles.insert(area_entity, info);
        return area_entity;
    }

    pub fn get_area_by_info(&self, id: &Entity, info: AreaInfo) -> Option<&Area2D> {
        match info.role {
            AreaRole::Physics => self
                .physical_colliders_2d
                .get(&info.parent)
                .unwrap()
                .get(id),
            _ => None,
        }
    }

    pub fn update_area_masks_and_layers(&mut self, id: &Entity, masks: u8, layers: u8) {
        if let Some(info) = self.area_roles.get(id) {
            if let Some(area) = match info.role {
                AreaRole::Physics => self
                    .physical_colliders_2d
                    .get_mut(&info.parent)
                    .unwrap()
                    .get_mut(id),
                _ => None,
            } {
                area.masks = masks;
                area.layers = layers;
            }
        }
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
            if let Some(body) = self.physics_bodies_2d.get(entity) {
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
                    position: [body.position[0], body.position[1]],
                    size: body.get_size(),
                    z_order: -body.position[1], // Sort top to bottom: lower y = drawn later
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

    pub fn clear_forces(&mut self) {
        for (_entity, body) in self.physics_bodies_2d.iter_mut() {
            body.force_accumulator = Vector2::new(0.0, 0.0);
        }
    }
}
