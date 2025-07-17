use std::collections::HashMap;

use crate::{
    components_systems::{physics_2d::PhysicsBody, Entity},
    world::World,
};

#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub acceleration: [f32; 2],
    pub size: [f32; 2],
}

pub fn transform_system_physics(world: &mut World, dt: f32) {
    for (entity, body) in world.physics_bodies_2d.iter_mut() {
        body.integrate(dt);
    }
}

pub fn transform_system_calculate_intended_position(
    world: &World,
    dt: f32,
) -> HashMap<Entity, PhysicsBody> {
    world
        .physics_bodies_2d
        .iter()
        .map(|(id, before)| {
            let updated = before.extrapolate(dt);
            (id.clone(), updated)
        })
        .collect()
}

fn apply_physics_to_transform(
    mut transform: TransformComponent,
    delta_time: f32,
    apply_drag: bool,
) -> TransformComponent {
    // Integrate acceleration
    transform.velocity[0] += transform.acceleration[0] * delta_time;
    transform.velocity[1] += transform.acceleration[1] * delta_time;

    // Clamp speed
    let speed = (transform.velocity[0].powi(2) + transform.velocity[1].powi(2)).sqrt();
    let max_speed = 10.0;
    if speed > max_speed {
        let scale = max_speed / speed;
        transform.velocity[0] *= scale;
        transform.velocity[1] *= scale;
    }

    // Apply velocity to position
    transform.position[0] += transform.velocity[0] * delta_time;
    transform.position[1] += transform.velocity[1] * delta_time;

    // Apply drag
    if apply_drag {
        let velocity_retention_per_second: f32 = 0.01;
        let drag = velocity_retention_per_second.powf(delta_time);
        transform.velocity[0] *= drag;
        transform.velocity[1] *= drag;
    }

    // Clear acceleration
    transform.acceleration = [0.0; 2];

    transform
}

pub fn transform_system_add_acceleration(world: &mut World, id: Entity, dx: f32, dy: f32) {
    if let Some(transform) = world.transforms_2d.get_mut(&id) {
        transform.acceleration[0] += dx;
        transform.acceleration[1] += dy;
    }
}

pub fn transform_system_redirect(
    world: &mut World,
    id: Entity,
    dx: f32,
    dy: f32,
    sep_x: f32,
    sep_y: f32,
    acceleration_mod: f32,
) {
    if let Some(transform) = world.transforms_2d.get_mut(&id) {
        transform.velocity[0] = dx;
        transform.velocity[1] = dy;

        transform.acceleration[0] = dx * acceleration_mod;
        transform.acceleration[1] = dy * acceleration_mod;

        transform.position[0] += sep_x;
        transform.position[1] += sep_y;
    }
}
