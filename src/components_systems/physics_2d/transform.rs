use std::collections::HashMap;

use cgmath::Vector2;

use crate::{
    components_systems::{physics_2d::PhysicsBody2D, Entity},
    world::World,
};

#[derive(Debug, Clone)]
pub struct Transform2D {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

impl Transform2D {
    pub fn get_size(&self) -> Vector2<f32> {
        self.scale.clone()
    }
}
/*
 *     pub fn get_size(&self) -> [f32; 2] {
        match &self.shape {
            Shape::Rectangle { half_extents } => [half_extents.x * 2.0, half_extents.y * 2.0],

            Shape::Circle { radius } => {
                let diameter = *radius * 2.0;
                [diameter, diameter]
            } /*
              Shape::Polygon { vertices } => {
                  // Compute AABB
                  let (min_x, max_x) = vertices.iter().map(|v| v.x).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| (min.min(x), max.max(x)));
                  let (min_y, max_y) = vertices.iter().map(|v| v.y).fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), y| (min.min(y), max.max(y)));
                  [(max_x - min_x), (max_y - min_y)]
              }
              */
              // Add other shapes as needed
        }
    }
*/

pub fn transform_system_physics(world: &mut World, dt: f32) {
    for (entity, body) in world.physics_bodies_2d.iter_mut() {
        let transform = world.transforms_2d.get_mut(&entity).unwrap();
        body.integrate(dt, transform);
    }
}

pub fn transform_system_calculate_intended_position(
    world: &World,
    dt: f32,
) -> HashMap<Entity, (PhysicsBody2D, Transform2D)> {
    world
        .physics_bodies_2d
        .iter()
        .map(|(id, before)| {
            let transform = world.transforms_2d.get(&id).unwrap();
            let updated = before.extrapolate(dt, transform);
            (id.clone(), updated)
        })
        .collect()
}
