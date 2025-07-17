use cgmath::Vector2;

#[derive(Debug, Clone)]
pub enum Shape {
    Circle { radius: f32 },
    // half extents (width/2, height/2) common in physics
    Rectangle { half_extents: Vector2<f32> },
    // Polygon { vertices: Vec<Vector2<f32>> }, // convex polygon, relative to center
    // could add Capsule, etc.
}

impl Shape {
    pub fn half_extents(&self) -> [f32; 2] {
        match *self {
            Shape::Rectangle { half_extents } => [half_extents.x, half_extents.y],
            Shape::Circle { radius } => [radius, radius],
        }
    }
}
