use std::collections::HashMap;

use crate::{
    components_systems::{
        physics_2d::{BodyType, PhysicsBody},
        Entity,
    },
    world::World,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ColliderComponent {
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub is_solid: bool,
    pub masks: u8,
    pub layers: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionInfo {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub a_size: [f32; 2],
    pub b_size: [f32; 2],
    pub next_pos_a: [f32; 2],
    pub next_pos_b: [f32; 2],
    pub velocity_a: [f32; 2],
    pub velocity_b: [f32; 2],
    pub normal: [f32; 2],
    pub penetration: f32,
}

pub fn collision_system(world: &World, next: &HashMap<Entity, PhysicsBody>) -> Vec<CollisionInfo> {
    let mut collisions = Vec::new();

    for (a, a_collider) in world.colliders_2d.iter() {
        for (b, b_collider) in world.colliders_2d.iter() {
            if a == b || a_collider.masks & b_collider.layers == 0 {
                // a doesn't care about b
                continue;
            }

            if let Some(a_next) = next.get(a) {
                if let Some(b_next) = next.get(b) {
                    if check_aabb_intersects(a_next, a_collider, b_next, b_collider) {
                        let a_half_size = [a_collider.size[0] / 2.0, a_collider.size[1] / 2.0];
                        let b_half_size = [b_collider.size[0] / 2.0, b_collider.size[1] / 2.0];

                        let a_min_x = a_next.position[0] - a_half_size[0];
                        let a_max_x = a_next.position[0] + a_half_size[0];
                        let a_min_y = a_next.position[1] - a_half_size[1];
                        let a_max_y = a_next.position[1] + a_half_size[1];

                        let b_min_x = b_next.position[0] - b_half_size[0];
                        let b_max_x = b_next.position[0] + b_half_size[0];
                        let b_min_y = b_next.position[1] - b_half_size[1];
                        let b_max_y = b_next.position[1] + b_half_size[1];

                        let overlap_x = f32::min(a_max_x, b_max_x) - f32::max(a_min_x, b_min_x);
                        let overlap_y = f32::min(a_max_y, b_max_y) - f32::max(a_min_y, b_min_y);
                        let normal = if overlap_x < overlap_y {
                            if a_next.position[0] < b_next.position[0] {
                                [1.0, 0.0] // A is left of B
                            } else {
                                [-1.0, 0.0] // A is right of B
                            }
                        } else {
                            if a_next.position[1] < b_next.position[1] {
                                [0.0, 1.0] // A is above B
                            } else {
                                [0.0, -1.0] // A is below B
                            }
                        };
                        let penetration = if overlap_x < overlap_y {
                            overlap_x
                        } else {
                            overlap_y
                        };

                        collisions.push(CollisionInfo {
                            entity_a: *a,
                            entity_b: *b,
                            a_size: a_next.get_size(),
                            b_size: b_next.get_size(),
                            next_pos_a: a_next.position.into(),
                            next_pos_b: b_next.position.into(),
                            velocity_a: a_next.velocity.into(),
                            velocity_b: b_next.velocity.into(),
                            penetration,
                            normal,
                        });
                    }
                }
            }
        }
    }
    collisions
}

pub fn check_aabb_intersects(
    a_transform: &PhysicsBody,
    a_collider: &ColliderComponent,
    b_transform: &PhysicsBody,
    b_collider: &ColliderComponent,
) -> bool {
    // Assuming position is center of entity and size is width/height
    let a_min = [
        a_transform.position[0] - a_collider.size[0] / 2.0,
        a_transform.position[1] - a_collider.size[1] / 2.0,
    ];
    let a_max = [
        a_transform.position[0] + a_collider.size[0] / 2.0,
        a_transform.position[1] + a_collider.size[1] / 2.0,
    ];

    let b_min = [
        b_transform.position[0] - b_collider.size[0] / 2.0,
        b_transform.position[1] - b_collider.size[1] / 2.0,
    ];
    let b_max = [
        b_transform.position[0] + b_collider.size[0] / 2.0,
        b_transform.position[1] + b_collider.size[1] / 2.0,
    ];

    let overlap_x = a_min[0] <= b_max[0] && a_max[0] >= b_min[0];
    let overlap_y = a_min[1] <= b_max[1] && a_max[1] >= b_min[1];

    overlap_x && overlap_y
}

pub fn resolve_collisions(world: &mut World, collisions: Vec<CollisionInfo>) {
    for col in collisions {
        let b_body = world.physics_bodies_2d.get(&col.entity_b).unwrap().clone();
        let a_body = world.physics_bodies_2d.get_mut(&col.entity_a).unwrap();
        let a_collider = world.colliders_2d.get(&col.entity_a).unwrap();
        let b_collider = world.colliders_2d.get(&col.entity_b).unwrap();

        if a_body.body_type == BodyType::Rigid
            && (b_body.body_type == BodyType::Static || b_body.body_type == BodyType::Rigid)
            && a_collider.masks & b_collider.layers != 0
        {
            // We should handle the collision
            // minimum translation vector
            let mtv = [
                col.normal[0] * col.penetration,
                col.normal[1] * col.penetration,
            ];
            a_body.position[0] = a_body.position[0] - mtv[0];
            a_body.position[1] = a_body.position[1] - mtv[1];
            // Simple slide: zero out the component of velocity along the normal
            let dot = a_body.velocity[0] * col.normal[0] + a_body.velocity[1] * col.normal[1];
            if dot < 0.0 {
                a_body.velocity[0] -= dot * col.normal[0];
                a_body.velocity[1] -= dot * col.normal[1];
            }
        }
    }
}
