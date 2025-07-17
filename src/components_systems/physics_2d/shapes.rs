use cgmath::Vector2;

#[derive(Debug, Clone)]
pub enum Shape {
    Circle { radius: f32 },
    // half extents (width/2, height/2) common in physics
    Rectangle { half_extents: Vector2<f32> },
    // Polygon { vertices: Vec<Vector2<f32>> }, // convex polygon, relative to center
    // could add Capsule, etc.
}
