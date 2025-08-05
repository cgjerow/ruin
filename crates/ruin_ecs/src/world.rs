use std::{collections::HashMap, hash::Hash};

use cgmath::Vector2;
use ruin_assets::{Handle, ImageTexture};

use crate::{
    physics_2d::{Area2D, Point2D, Shape2D},
    ActionStateComponent, AnimationComponent, Entity, FlipComponent, HealthComponent, Transform2D,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AreaRole {
    Physics,
    Hitbox,
    Hurtbox,
    #[allow(unused)]
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AreaInfo {
    pub role: AreaRole,
    pub parent: Entity,
}

#[derive(Debug, Clone, Copy)]
struct ParentAreaInfo {
    pub masks_superset: u8,
    pub layers_superset: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldDebug {
    pub enabled: bool,
    pub show_hitboxes: bool,
    pub show_hurtboxes: bool,
    pub show_colliders: bool,
}

#[derive(Debug, Clone)]
pub struct World {
    next_id: u32,
    pub flips: HashMap<Entity, FlipComponent>,
    pub health_bars: HashMap<Entity, HealthComponent>,
    pub animations: HashMap<Entity, AnimationComponent>,
    pub transforms_2d: HashMap<Entity, Transform2D>,
    pub action_states: HashMap<Entity, ActionStateComponent>,
    pub physical_colliders_2d: HashMap<Entity, HashMap<Entity, Area2D>>,
    pub hitboxes_2d: HashMap<Entity, HashMap<Entity, Area2D>>,
    pub hurtboxes_2d: HashMap<Entity, HashMap<Entity, Area2D>>,
    pub area_roles: HashMap<Entity, AreaInfo>,
    pub debug: WorldDebug,

    // keep this concept hidden for now.
    // interactions should take place through our getters/setters
    parent_area_info: HashMap<Entity, HashMap<AreaRole, ParentAreaInfo>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            health_bars: HashMap::new(),
            transforms_2d: HashMap::new(),
            action_states: HashMap::new(),
            animations: HashMap::new(),
            physical_colliders_2d: HashMap::new(),
            hitboxes_2d: HashMap::new(),
            hurtboxes_2d: HashMap::new(),
            area_roles: HashMap::new(),
            flips: HashMap::new(),
            parent_area_info: HashMap::new(),
            debug: WorldDebug {
                // this lowers frame rate.
                // use with minimal objs in scene
                enabled: false,
                show_hitboxes: true,
                show_hurtboxes: true,
                show_colliders: true,
            },
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        let entity = self.next_id;
        self.next_id += 1;
        return entity;
    }

    fn get_all_areas_by_info(&self, info: AreaInfo) -> HashMap<Entity, Area2D> {
        match info.role {
            AreaRole::Physics => self
                .physical_colliders_2d
                .get(&info.parent)
                .cloned()
                .unwrap_or_else(HashMap::new),
            _ => HashMap::new(),
        }
    }

    pub fn masks_overlap_layers(&self, a_info: AreaInfo, b_info: AreaInfo) -> u8 {
        if a_info.parent == b_info.parent {
            return 0;
        }
        let a = self
            .parent_area_info
            .get(&a_info.parent)
            .unwrap()
            .get(&a_info.role)
            .unwrap();
        let b = self
            .parent_area_info
            .get(&b_info.parent)
            .unwrap()
            .get(&b_info.role)
            .unwrap();

        return a.masks_superset & b.layers_superset;
    }

    pub fn layers_superset(&self, info: &AreaInfo) -> u8 {
        self.parent_area_info
            .get(&info.parent)
            .unwrap()
            .get(&info.role)
            .unwrap()
            .layers_superset
    }

    pub fn masks_superset(&self, info: &AreaInfo) -> u8 {
        self.parent_area_info
            .get(&info.parent)
            .unwrap()
            .get(&info.role)
            .unwrap()
            .masks_superset
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
        self.area_roles.insert(
            area_entity,
            AreaInfo {
                role: info.role,
                parent: info.parent.clone(),
            },
        );

        self.update_parent_area_info(info);

        return area_entity;
    }

    fn update_parent_area_info(&mut self, info: AreaInfo) {
        let all_areas = self.get_all_areas_by_info(info);
        let combined_masks = all_areas
            .values()
            .filter(|area| area.active)
            .fold(0u8, |acc, area| acc | area.masks);
        let combined_layers = all_areas
            .values()
            .filter(|area| area.active)
            .fold(0u8, |acc, area| acc | area.layers);
        self.parent_area_info
            .entry(info.parent) // Get entry for outer map
            .or_insert_with(HashMap::new) // Insert new inner HashMap if missing
            .insert(
                info.role.clone(),
                ParentAreaInfo {
                    masks_superset: combined_masks,
                    layers_superset: combined_layers,
                },
            );
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

    pub fn toggle_area(&mut self, id: &Entity, active: bool) {
        if let Some(info) = self.area_roles.get(id) {
            if let Some(area) = match info.role {
                AreaRole::Physics => self
                    .physical_colliders_2d
                    .get_mut(&info.parent)
                    .unwrap()
                    .get_mut(id),
                _ => None,
            } {
                area.active = active;
            }
            self.update_parent_area_info(*info);
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
            self.update_parent_area_info(*info);
        }
    }

    pub fn update_positions(&mut self, positions: HashMap<Entity, Point2D>) {
        for (entity, point) in positions {
            if let Some(transform) = self.transforms_2d.get_mut(&entity) {
                transform.position = Vector2 {
                    x: point.x,
                    y: point.y,
                };
            }
        }
    }

    pub fn extract_render_queue_2d(&self) -> RenderQueue2D {
        let mut transparent = Vec::new();
        let mut opaque = Vec::new();

        for (entity, animation) in self.animations.iter() {
            if let Some(transform) = self.transforms_2d.get(entity) {
                let uv_coords = animation.current_frame.uv_coords;
                let action_animation = &animation.animations[&self
                    .action_states
                    .get(&entity)
                    .expect("Animation not found")
                    .state];

                let tmp = RenderElement2D {
                    shape: &transform.shape,
                    position: transform.position.into(),
                    size: transform.scale.into(),
                    z_order: -transform.position[1], // Sort top to bottom: lower y = drawn later
                    image_texture: action_animation.sprite_sheet_id,
                    uv_coords,
                };

                if action_animation.is_transparent {
                    transparent.push(tmp);
                } else {
                    opaque.push(tmp);
                }
            }
        }

        RenderQueue2D {
            transparent,
            opaque,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderElement2D<'a> {
    pub shape: &'a Shape2D,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub z_order: f32, // for Y-based sorting (e.g., lower y = drawn on top)
    pub image_texture: Handle<ImageTexture>,
    pub uv_coords: [[f32; 2]; 4],
}

#[derive(Debug, Clone)]
pub struct RenderQueue2D<'a> {
    pub transparent: Vec<RenderElement2D<'a>>,
    pub opaque: Vec<RenderElement2D<'a>>,
}
