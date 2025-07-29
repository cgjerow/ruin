use cgmath::Vector2;

use crate::components_systems::physics_2d::Shape2D;

#[derive(Debug, Clone)]
pub struct Area2D {
    pub shape: Shape2D,
    pub offset: Vector2<f32>,
    pub layers: u8,
    pub masks: u8,
    pub active: bool, // similar to sleep
}
