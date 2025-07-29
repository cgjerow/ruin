use cgmath::{InnerSpace, Vector2};

#[derive(Debug, Clone)]
pub enum Shape2D {
    Circle { radius: f32 },
    // half extents (width/2, height/2) common in physics
    Rectangle { half_extents: Vector2<f32> },
    // Polygon { vertices: Vec<Vector2<f32>> }, // convex polygon, relative to center
    // could add Capsule, etc.
}

impl Shape2D {
    pub fn half_extents(&self) -> Vector2<f32> {
        match *self {
            Shape2D::Rectangle { half_extents } => half_extents,
            Shape2D::Circle { radius } => Vector2 {
                x: radius,
                y: radius,
            },
        }
    }

    pub fn scale(&self, scale: Vector2<f32>) -> Self {
        match *self {
            Shape2D::Rectangle { half_extents } => Shape2D::Rectangle {
                half_extents: Vector2 {
                    x: half_extents.x * scale.x,
                    y: half_extents.y * scale.y,
                },
            },
            Shape2D::Circle { radius } => Shape2D::Circle {
                radius: radius * scale.magnitude(),
            },
        }
    }
}
