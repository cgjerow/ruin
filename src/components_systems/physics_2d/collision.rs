use std::collections::HashMap;

use crate::{
    components_systems::{
        physics_2d::{Area2D, BodyType, PhysicsBody2D, Transform2D},
        Entity,
    },
    world::{AreaInfo, AreaRole, World},
};

#[derive(Debug, Clone, Copy)]
pub struct CollisionInfo {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub a_area_collider: Entity,
    pub b_area_collider: Entity,
    pub a_size: [f32; 2],
    pub b_size: [f32; 2],
    pub next_pos_a: [f32; 2],
    pub next_pos_b: [f32; 2],
    pub velocity_a: [f32; 2],
    pub velocity_b: [f32; 2],
    pub normal: [f32; 2],
    pub penetration: f32,
}

pub fn collision_system(
    world: &World,
    next: &HashMap<Entity, (PhysicsBody2D, Transform2D)>,
) -> Vec<CollisionInfo> {
    let mut collisions = Vec::new();

    for (a_parent, a_map) in world.physical_colliders_2d.iter() {
        for (a_area_id, a_collider) in a_map.iter() {
            for (b_parent, b_map) in world.physical_colliders_2d.iter() {
                if world.masks_overlap_layers(
                    AreaInfo {
                        parent: a_parent.clone(),
                        role: AreaRole::Physics,
                    },
                    AreaInfo {
                        parent: b_parent.clone(),
                        role: AreaRole::Physics,
                    },
                ) == 0
                {
                    continue;
                }

                for (b_area_id, b_collider) in b_map.iter() {
                    // Skip self-collision or uninteresting interactions
                    if a_area_id == b_area_id || a_collider.masks & b_collider.layers == 0 {
                        continue;
                    }

                    // Get predicted next transforms for each parent
                    if let (Some(a_next), Some(b_next)) = (next.get(a_parent), next.get(b_parent)) {
                        if check_aabb_intersects(&a_next.1, a_collider, &b_next.1, b_collider) {
                            let a_half_size = a_collider.shape.half_extents();
                            let b_half_size = b_collider.shape.half_extents();

                            let a_min_x = a_next.1.position[0] - a_half_size[0];
                            let a_max_x = a_next.1.position[0] + a_half_size[0];
                            let a_min_y = a_next.1.position[1] - a_half_size[1];
                            let a_max_y = a_next.1.position[1] + a_half_size[1];

                            let b_min_x = b_next.1.position[0] - b_half_size[0];
                            let b_max_x = b_next.1.position[0] + b_half_size[0];
                            let b_min_y = b_next.1.position[1] - b_half_size[1];
                            let b_max_y = b_next.1.position[1] + b_half_size[1];

                            let overlap_x = f32::min(a_max_x, b_max_x) - f32::max(a_min_x, b_min_x);
                            let overlap_y = f32::min(a_max_y, b_max_y) - f32::max(a_min_y, b_min_y);

                            let normal = if overlap_x < overlap_y {
                                if a_next.1.position[0] < b_next.1.position[0] {
                                    [1.0, 0.0]
                                } else {
                                    [-1.0, 0.0]
                                }
                            } else {
                                if a_next.1.position[1] < b_next.1.position[1] {
                                    [0.0, 1.0]
                                } else {
                                    [0.0, -1.0]
                                }
                            };

                            let penetration = if overlap_x < overlap_y {
                                overlap_x
                            } else {
                                overlap_y
                            };

                            collisions.push(CollisionInfo {
                                entity_a: *a_parent,
                                entity_b: *b_parent,
                                a_area_collider: *a_area_id,
                                b_area_collider: *b_area_id,
                                a_size: a_next.1.get_size().into(),
                                b_size: b_next.1.get_size().into(),
                                next_pos_a: a_next.1.position.into(),
                                next_pos_b: b_next.1.position.into(),
                                velocity_a: a_next.0.velocity.into(),
                                velocity_b: b_next.0.velocity.into(),
                                penetration,
                                normal,
                            });
                        }
                    }
                }
            }
        }
        // println!("---- A Outer Loop: {:.6}", a_1.elapsed().as_secs_f64());
    }
    collisions
}

pub fn check_aabb_intersects(
    a_transform: &Transform2D,
    a_collider: &Area2D,
    b_transform: &Transform2D,
    b_collider: &Area2D,
) -> bool {
    // Assuming position is center of entity and size is width/height
    let a_min = [
        a_transform.position[0] - a_collider.shape.half_extents()[0],
        a_transform.position[1] - a_collider.shape.half_extents()[1],
    ];
    let a_max = [
        a_transform.position[0] + a_collider.shape.half_extents()[0],
        a_transform.position[1] + a_collider.shape.half_extents()[1],
    ];

    let b_min = [
        b_transform.position[0] - b_collider.shape.half_extents()[0],
        b_transform.position[1] - b_collider.shape.half_extents()[1],
    ];
    let b_max = [
        b_transform.position[0] + b_collider.shape.half_extents()[0],
        b_transform.position[1] + b_collider.shape.half_extents()[1],
    ];

    let overlap_x = a_min[0] <= b_max[0] && a_max[0] >= b_min[0];
    let overlap_y = a_min[1] <= b_max[1] && a_max[1] >= b_min[1];

    overlap_x && overlap_y
}

pub fn resolve_collisions(world: &mut World, collisions: Vec<CollisionInfo>) {
    for col in collisions {
        let a_collider = world
            .get_area_by_info(
                &col.a_area_collider,
                AreaInfo {
                    parent: col.entity_a,
                    role: AreaRole::Physics,
                },
            )
            .unwrap()
            .clone();
        let b_collider = world
            .get_area_by_info(
                &col.b_area_collider,
                AreaInfo {
                    parent: col.entity_b,
                    role: AreaRole::Physics,
                },
            )
            .unwrap()
            .clone();

        let b_body = world.physics_bodies_2d.get(&col.entity_b).unwrap().clone();
        let a_body = world.physics_bodies_2d.get_mut(&col.entity_a).unwrap();
        let a_pos = world.transforms_2d.get_mut(&col.entity_a).unwrap();

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

            if mtv[0] > 0.5 || mtv[1] > 0.5 {
                a_pos.position[0] = a_pos.position[0] - mtv[0];
                a_pos.position[1] = a_pos.position[1] - mtv[1];
            }

            // Simple slide: zero out the component of velocity along the normal
            let dot = a_body.velocity[0] * col.normal[0] + a_body.velocity[1] * col.normal[1];
            if dot != 0.0 {
                a_body.velocity[0] -= dot * col.normal[0];
                a_body.velocity[1] -= dot * col.normal[1];
            }
        }
    }
}
