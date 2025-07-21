use cgmath::Vector2;

use crate::components_systems::physics_2d::Shape;

#[derive(Debug, Clone)]
pub struct Area2D {
    pub shape: Shape,
    pub offset: Vector2<f32>,
    pub layers: u8,
    pub masks: u8,
}
