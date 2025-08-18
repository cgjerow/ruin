use cgmath::{InnerSpace, Vector2};
use ruin_bitmaps::masks_overlap_layers;

use crate::physics_2d::{
    body_2d::{NormalizeZero, AABB},
    Body2D, BodyType2D, CollisionPair, CollisionResolver, Vector2D,
};

pub struct SimpleCollideAndSlideCollisionResolver {
    slop: f32,
}

impl SimpleCollideAndSlideCollisionResolver {
    pub fn new(slop: f32) -> Self {
        SimpleCollideAndSlideCollisionResolver { slop }
    }
}

impl CollisionResolver for SimpleCollideAndSlideCollisionResolver {
    fn resolve(&mut self, bodies: &mut Vec<Body2D>, collisions: &Vec<CollisionPair>) {
        for pair in collisions {
            let (a_idx, b_idx) = (pair.a, pair.b);
            let (a, b) = {
                let (left, right) = bodies.split_at_mut(std::cmp::max(a_idx, b_idx));
                if a_idx < b_idx {
                    (&mut left[a_idx], &mut right[0])
                } else {
                    (&mut right[0], &mut left[b_idx])
                }
            };

            // Skip if both are Kinematic or Trigger
            if matches!(a.body_type(), BodyType2D::Kinematic | BodyType2D::Trigger)
                && matches!(b.body_type(), BodyType2D::Kinematic | BodyType2D::Trigger)
            {
                continue;
            }

            for a_aabb in &a.aabbs {
                for b_aabb in &b.aabbs {
                    if masks_overlap_layers(a_aabb.masks, b_aabb.layers)
                        && a_aabb.aabb.overlaps(&b_aabb.aabb)
                    {
                        if let Some(overlap) = compute_mtv(&a_aabb.aabb, &b_aabb.aabb) {
                            let penetration = overlap.magnitude();
                            if penetration <= self.slop {
                                continue; // Ignore very small penetrations
                            }

                            let normal = overlap.normalize_to_zero();
                            let mtv = normal * penetration;

                            match (&a.body_type(), &b.body_type()) {
                                (BodyType2D::Rigid, BodyType2D::Rigid) => {
                                    a.position -= mtv * 0.5;
                                    b.position += mtv * 0.5;

                                    let dot_a = a.velocity.dot(normal);
                                    if dot_a < 0.0 {
                                        a.velocity -= normal * dot_a;
                                    }

                                    let dot_b = b.velocity.dot(-normal);
                                    if dot_b < 0.0 {
                                        b.velocity -= (-normal) * dot_b;
                                    }
                                }

                                (BodyType2D::Rigid, BodyType2D::Static) => {
                                    a.position -= mtv;

                                    let dot = a.velocity.dot(normal);
                                    if dot < 0.0 {
                                        a.velocity -= normal * dot;
                                    }
                                }

                                (BodyType2D::Static, BodyType2D::Rigid) => {
                                    b.position += mtv;

                                    let dot = b.velocity.dot(-normal);
                                    if dot < 0.0 {
                                        b.velocity -= (-normal) * dot;
                                    }
                                }

                                _ => {
                                    // Kinematic and Trigger logic can go here
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn compute_mtv(a: &AABB, b: &AABB) -> Option<Vector2<f32>> {
    let a_min = a.min;
    let a_max = a.max;
    let b_min = b.min;
    let b_max = b.max;

    let dx1 = b_max.x - a_min.x; // overlap if b is to the right
    let dx2 = a_max.x - b_min.x; // overlap if b is to the left
    let dy1 = b_max.y - a_min.y; // overlap if b is above
    let dy2 = a_max.y - b_min.y; // overlap if b is below

    let overlap_x = dx1.min(dx2);
    let overlap_y = dy1.min(dy2);

    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return None; // no actual overlap
    }

    // Resolve along the smaller axis (fastest way out)
    if overlap_x < overlap_y {
        let direction = if dx1 < dx2 { -1.0 } else { 1.0 };
        Some(Vector2D::new(direction * overlap_x, 0.0))
    } else {
        let direction = if dy1 < dy2 { -1.0 } else { 1.0 };
        Some(Vector2D::new(0.0, direction * overlap_y))
    }
}
