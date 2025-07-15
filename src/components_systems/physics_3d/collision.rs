use std::collections::HashMap;

use crate::{
    components_systems::{physics_3d::TransformComponent, Entity},
    world::World,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ColliderComponent {
    pub offset: [f32; 3],
    pub size: [f32; 3],
    pub is_solid: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionInfo {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub a_size: [f32; 3],
    pub b_size: [f32; 3],
    pub next_pos_a: [f32; 3],
    pub next_pos_b: [f32; 3],
    pub velocity_a: [f32; 3],
    pub velocity_b: [f32; 3],
    pub normal: [f32; 3],
}

pub fn collision_system(
    world: &World,
    next_transforms: &HashMap<Entity, TransformComponent>,
) -> Vec<CollisionInfo> {
    let mut collisions = Vec::new();

    for (entity, next_transform) in next_transforms.iter() {
        if let Some(collider) = world.colliders_3d.get(entity) {
            for (other_entity, other_collider) in world.colliders_3d.iter() {
                if entity == other_entity {
                    continue;
                }

                if let Some(other_next_transform) = next_transforms.get(other_entity) {
                    if is_colliding_3d(
                        next_transform,
                        collider,
                        other_next_transform,
                        other_collider,
                    ) {
                        let dx = next_transform.position[0] - other_next_transform.position[0];
                        let dy = next_transform.position[1] - other_next_transform.position[1];
                        let dz = next_transform.position[2] - other_next_transform.position[2];
                        let mag = (dx * dx + dy * dy + dz * dz).sqrt();

                        let normal = if mag != 0.0 {
                            [dx / mag, dy / mag, dz / mag]
                        } else {
                            [0.0, 0.0, 0.0] // exact overlap
                        };

                        collisions.push(CollisionInfo {
                            entity_a: *entity,
                            entity_b: *other_entity,
                            a_size: next_transform.size,
                            b_size: other_next_transform.size,
                            next_pos_a: next_transform.position,
                            next_pos_b: other_next_transform.position,
                            velocity_a: next_transform.velocity,
                            velocity_b: other_next_transform.velocity,
                            normal,
                        });
                    }
                }
            }
        }
    }

    collisions
}

pub fn is_colliding_3d(
    a_transform: &TransformComponent,
    a_collider: &ColliderComponent,
    b_transform: &TransformComponent,
    b_collider: &ColliderComponent,
) -> bool {
    // Assuming position is center of entity and size is width/height/depth
    let a_min = [
        a_transform.position[0] - a_collider.size[1] / 2.0,
        a_transform.position[1] - a_collider.size[1] / 2.0,
        a_transform.position[2] - a_collider.size[2] / 2.0,
    ];
    let a_max = [
        a_transform.position[0] + a_collider.size[0] / 2.0,
        a_transform.position[1] + a_collider.size[1] / 2.0,
        a_transform.position[2] + a_collider.size[2] / 2.0,
    ];

    let b_min = [
        b_transform.position[0] - b_collider.size[0] / 2.0,
        b_transform.position[1] - b_collider.size[1] / 2.0,
        b_transform.position[2] - b_collider.size[2] / 2.0,
    ];
    let b_max = [
        b_transform.position[0] + b_collider.size[0] / 2.0,
        b_transform.position[1] + b_collider.size[1] / 2.0,
        b_transform.position[2] + b_collider.size[2] / 2.0,
    ];

    // Check for overlap on all 3 axes (X, Y, Z)
    let overlap_x = a_min[0] <= b_max[0] && a_max[0] >= b_min[0];
    let overlap_y = a_min[1] <= b_max[1] && a_max[1] >= b_min[1];
    let overlap_z = a_min[2] <= b_max[2] && a_max[2] >= b_min[2];

    overlap_x && overlap_y && overlap_z
}
