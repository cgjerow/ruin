use crate::physics_2d::{body_2d::Index, Body2D};

#[derive(Debug, Clone)]
pub struct CollisionPair {
    pub a: Index,
    pub b: Index,
}

pub trait CollisionDetector {
    fn broad_phase(&mut self, bodies: &Vec<Body2D>) -> Vec<CollisionPair>;
    fn narrow_phase(&mut self, broad_phase_results: &Vec<CollisionPair>) -> Vec<CollisionPair>;
}

pub trait CollisionResolver {
    fn resolve(&mut self, bodies: &mut Vec<Body2D>, collisions: &Vec<CollisionPair>);
}
