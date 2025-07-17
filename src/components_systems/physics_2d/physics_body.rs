use cgmath::Vector2;
use std::u8;

use crate::components_systems::physics_2d::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Static,
    Rigid,
    Kinematic,
    Trigger,
}

impl From<u8> for BodyType {
    fn from(value: u8) -> Self {
        match value {
            0 => BodyType::Rigid,
            1 => BodyType::Static,
            2 => BodyType::Kinematic,
            3 => BodyType::Trigger,
            _ => BodyType::Rigid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub body_type: BodyType,
    /// Position is center of mass / shape
    pub position: Vector2<f32>,
    /// Velocity in units per second
    pub velocity: Vector2<f32>,
    pub force_accumulator: Vector2<f32>,
    pub mass: f32,
    pub shape: Shape,
}

const DEADZONE: f32 = 0.00001;

impl PhysicsBody {
    /// Applies a force to this body, if allowed by body type
    pub fn apply_force(&mut self, force: Vector2<f32>) {
        match self.body_type {
            BodyType::Rigid | BodyType::Kinematic => {
                // Kinematic often ignores forces, but allowing here for example
                self.force_accumulator += force;
            }
            BodyType::Static | BodyType::Trigger => {
                // No effect on static or trigger bodies
            }
        }
    }

    pub fn get_size(&self) -> [f32; 2] {
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

    /// Applies an impulse: instantaneous velocity change, bypassing forces
    /// Useful for collisions, knockbacks, etc.
    pub fn apply_impulse(&mut self, impulse: Vector2<f32>) {
        // Usually impulses don't affect static, kinematic, trigger bodies
        if self.body_type == BodyType::Rigid {
            // impulse = Δmomentum = mass * Δvelocity → Δvelocity = impulse / mass
            self.velocity += impulse / self.mass;
            // don't want accumulated forces to conflict with impulse
            self.force_accumulator = Vector2::new(0.0, 0.0);
        }
    }

    /// Physics integration step: update velocity and position based on accumulated forces and dt
    pub fn integrate(&mut self, dt: f32) {
        let extrapolated = self.extrapolate(dt);
        self.velocity = Self::apply_deadzone(extrapolated.velocity);
        self.position = extrapolated.position;
        self.force_accumulator = extrapolated.force_accumulator;
    }

    fn apply_deadzone(v: Vector2<f32>) -> Vector2<f32> {
        let mut w = Vector2::new(0.0, 0.0);
        for i in 0..2 {
            if v[i].abs() > DEADZONE {
                w[i] = v[i];
            }
        }
        return w;
    }

    pub fn extrapolate(&self, dt: f32) -> PhysicsBody {
        let mut physics_body = self.clone();
        if self.body_type != BodyType::Rigid {
            return physics_body;
        }
        let acceleration = physics_body.force_accumulator / physics_body.mass;
        physics_body.velocity += acceleration * dt;
        physics_body.position += physics_body.velocity * dt;
        return physics_body;
    }
}
