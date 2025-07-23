use cgmath::Vector2;
use std::u8;

use crate::components_systems::physics_2d::{Shape, Transform2D};

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
pub struct PhysicsBody2D {
    pub body_type: BodyType,
    /// Velocity in units per second
    pub velocity: Vector2<f32>,
    pub force_accumulator: Vector2<f32>,
    pub mass: f32,
}

const DEADZONE: f32 = 0.00001;

impl PhysicsBody2D {
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
    pub fn integrate(&mut self, dt: f32, transform: &mut Transform2D) {
        let extrapolated = self.extrapolate(dt, transform);
        self.velocity = Self::apply_deadzone(extrapolated.0.velocity);
        self.force_accumulator = extrapolated.0.force_accumulator;
        transform.position = extrapolated.1.position;
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

    pub fn extrapolate(&self, dt: f32, t: &Transform2D) -> (PhysicsBody2D, Transform2D) {
        let mut physics_body = self.clone();
        let mut transform = t.clone();
        if self.body_type != BodyType::Rigid {
            return (physics_body, transform);
        }

        let acceleration = physics_body.force_accumulator / physics_body.mass;
        physics_body.velocity += acceleration * dt;
        transform.position += physics_body.velocity * dt;
        return (physics_body, transform);
    }
}
