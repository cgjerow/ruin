use crate::physics_2d::{body_2d::Index, Body2D, BodyType2D, Point2D};

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
