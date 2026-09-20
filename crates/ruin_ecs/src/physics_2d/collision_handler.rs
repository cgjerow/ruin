use crate::physics_2d::{body_2d::Index, Body2D, BodyType2D, Point2D};

/// Check if a body's AABB center is within range of the center point.
pub fn body_in_range(body: &Body2D, center: Point2D, range: f32) -> bool {
    let body_center = body.aabb_superset.center();
    let dx = (body_center.x - center.x).abs();
    let dy = (body_center.y - center.y).abs();
    dx <= range && dy <= range
}

/// Check if a body is terrain (static or kinematic).
pub fn is_terrain(body: &Body2D) -> bool {
    matches!(body.body_type(), BodyType2D::Static | BodyType2D::Kinematic)
}

#[derive(Debug, Clone)]
pub struct CollisionPair {
    pub a: Index,
    pub b: Index,
}

pub trait CollisionDetector {
    fn update_player_position(&mut self, position: Point2D, physics_range: f32);
    fn broad_phase(&mut self, bodies: &Vec<Body2D>, center: Point2D, range: f32) -> Vec<CollisionPair>;
    fn narrow_phase(&mut self, broad_phase_results: &Vec<CollisionPair>) -> Vec<CollisionPair>;
}

pub trait CollisionResolver {
    fn resolve(&mut self, bodies: &mut Vec<Body2D>, collisions: &Vec<CollisionPair>);
}
