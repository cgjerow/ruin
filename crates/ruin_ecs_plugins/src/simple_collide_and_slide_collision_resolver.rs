use bitflags::bitflags;
use cgmath::{InnerSpace, Vector2};
use ruin_bitmaps::masks_overlap_layers;

use ruin_ecs::physics_2d::{
    Body2D, BodyType2D, CollisionPair, CollisionResolver, NormalizeZero, Vector2D, AABB,
};

bitflags! {
    #[derive(Clone)]
    struct AnchorDir: u8 {
        const POS_X = 0b0001;
        const NEG_X = 0b0010;
        const POS_Y = 0b0100;
        const NEG_Y = 0b1000;
    }
}

pub struct SimpleCollideAndSlideCollisionResolver {
    slop: f32,
}

impl SimpleCollideAndSlideCollisionResolver {
    pub fn new(slop: f32) -> Self {
        Self { slop }
    }

    fn resolve_pair(
        &self,
        bodies: &mut Vec<Body2D>,
        pair: &CollisionPair,
        anchored: &mut [AnchorDir],
    ) {
        let (a_idx, b_idx) = (pair.a, pair.b);
        let (left, right) = bodies.split_at_mut(std::cmp::max(a_idx, b_idx));
        let (a, b) = if a_idx < b_idx {
            (&mut left[a_idx], &mut right[0])
        } else {
            (&mut right[0], &mut left[b_idx])
        };

        for a_aabb in &a.aabbs {
            for b_aabb in &b.aabbs {
                if !masks_overlap_layers(a_aabb.masks, b_aabb.layers) {
                    continue;
                }
                if let Some(overlap) = compute_mtv(&a_aabb.aabb, &b_aabb.aabb) {
                    let penetration = overlap.magnitude();
                    if penetration <= self.slop {
                        continue;
                    }

                    let normal = overlap.normalize_to_zero();
                    let mtv = normal * penetration;

                    match (a.body_type(), b.body_type()) {
                        (BodyType2D::Rigid, BodyType2D::Static) => {
                            a.position -= mtv;

                            // directional anchoring
                            if mtv.x > 0.0 {
                                anchored[a_idx] |= AnchorDir::NEG_X;
                            }
                            if mtv.x < 0.0 {
                                anchored[a_idx] |= AnchorDir::POS_X;
                            }
                            if mtv.y > 0.0 {
                                anchored[a_idx] |= AnchorDir::NEG_Y;
                            }
                            if mtv.y < 0.0 {
                                anchored[a_idx] |= AnchorDir::POS_Y;
                            }

                            let dot = a.velocity.dot(normal);
                            if dot < 0.0 {
                                a.velocity -= normal * dot;
                            }
                        }
                        (BodyType2D::Static, BodyType2D::Rigid) => {
                            b.position += mtv;

                            if mtv.x > 0.0 {
                                anchored[b_idx] |= AnchorDir::POS_X;
                            }
                            if mtv.x < 0.0 {
                                anchored[b_idx] |= AnchorDir::NEG_X;
                            }
                            if mtv.y > 0.0 {
                                anchored[b_idx] |= AnchorDir::POS_Y;
                            }
                            if mtv.y < 0.0 {
                                anchored[b_idx] |= AnchorDir::NEG_Y;
                            }

                            let dot = b.velocity.dot(-normal);
                            if dot < 0.0 {
                                b.velocity -= (-normal) * dot;
                            }
                        }
                        (BodyType2D::Rigid, BodyType2D::Rigid) => {
                            // directional constraints: zero out components blocked by anchored flags
                            let mut move_a = mtv * 0.5;
                            let mut move_b = mtv * 0.5;

                            if mtv.x > 0.0 && anchored[a_idx].contains(AnchorDir::POS_X) {
                                move_a.x = 0.0;
                            }
                            if mtv.x < 0.0 && anchored[a_idx].contains(AnchorDir::NEG_X) {
                                move_a.x = 0.0;
                            }
                            if mtv.y > 0.0 && anchored[a_idx].contains(AnchorDir::POS_Y) {
                                move_a.y = 0.0;
                            }
                            if mtv.y < 0.0 && anchored[a_idx].contains(AnchorDir::NEG_Y) {
                                move_a.y = 0.0;
                            }

                            if mtv.x > 0.0 && anchored[b_idx].contains(AnchorDir::POS_X) {
                                move_b.x = 0.0;
                            }
                            if mtv.x < 0.0 && anchored[b_idx].contains(AnchorDir::NEG_X) {
                                move_b.x = 0.0;
                            }
                            if mtv.y > 0.0 && anchored[b_idx].contains(AnchorDir::POS_Y) {
                                move_b.y = 0.0;
                            }
                            if mtv.y < 0.0 && anchored[b_idx].contains(AnchorDir::NEG_Y) {
                                move_b.y = 0.0;
                            }

                            a.position -= move_a;
                            b.position += move_b;

                            // velocity clamping
                            let dot_a = a.velocity.dot(normal);
                            if dot_a < 0.0 {
                                a.velocity -= normal * dot_a;
                            }

                            let dot_b = b.velocity.dot(-normal);
                            if dot_b < 0.0 {
                                b.velocity -= (-normal) * dot_b;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl CollisionResolver for SimpleCollideAndSlideCollisionResolver {
    fn resolve(&mut self, bodies: &mut Vec<Body2D>, collisions: &Vec<CollisionPair>) {
        if bodies.is_empty() || collisions.is_empty() {
            return;
        }

        let mut anchored = vec![AnchorDir::empty(); bodies.len()];
        let mut static_pairs = Vec::new();
        let mut dynamic_pairs = Vec::new();

        for pair in collisions {
            let a_type = bodies[pair.a].body_type();
            let b_type = bodies[pair.b].body_type();

            if matches!(
                (a_type, b_type),
                (BodyType2D::Rigid, BodyType2D::Static) | (BodyType2D::Static, BodyType2D::Rigid)
            ) {
                static_pairs.push(pair);
            } else {
                dynamic_pairs.push(pair);
            }
        }

        // resolve static collisions first
        for pair in static_pairs {
            self.resolve_pair(bodies, pair, &mut anchored);
        }

        // then resolve dynamic-dynamic collisions
        for pair in dynamic_pairs {
            self.resolve_pair(bodies, pair, &mut anchored);
        }
    }
}

// compute_mtv unchanged
fn compute_mtv(a: &AABB, b: &AABB) -> Option<Vector2<f32>> {
    let a_min = a.min;
    let a_max = a.max;
    let b_min = b.min;
    let b_max = b.max;

    let dx1 = b_max.x - a_min.x;
    let dx2 = a_max.x - b_min.x;
    let dy1 = b_max.y - a_min.y;
    let dy2 = a_max.y - b_min.y;

    let overlap_x = dx1.min(dx2);
    let overlap_y = dy1.min(dy2);

    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return None;
    }

    if overlap_x < overlap_y {
        let direction = if dx1 < dx2 { -1.0 } else { 1.0 };
        Some(Vector2D::new(direction * overlap_x, 0.0))
    } else {
        let direction = if dy1 < dy2 { -1.0 } else { 1.0 };
        Some(Vector2D::new(0.0, direction * overlap_y))
    }
}
