use cgmath::Vector2;

use crate::physics_2d::Shape2D;

#[derive(Debug, Clone)]
pub struct Transform2D {
    pub position: Vector2<f32>,
    pub shape: Shape2D,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

impl Transform2D {
    pub fn get_scale_abs(&self) -> Vector2<f32> {
        Vector2::new(self.scale.x.abs(), self.scale.y.abs())
    }

    pub fn get_size(&self) -> [f32; 2] {
        match &self.shape {
            Shape2D::Rectangle { half_extents } => [
                half_extents.x * 2.0 * self.scale.x.abs(),
                half_extents.y * 2.0 * self.scale.y.abs(),
            ],

            Shape2D::Circle { radius } => {
                let diameter = *radius * 2.0;
                [diameter * self.scale.x.abs(), diameter * self.scale.y.abs()]
            } // Add other shapes as needed
        }
    }
}
