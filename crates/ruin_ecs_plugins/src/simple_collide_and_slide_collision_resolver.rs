use bitflags::bitflags;
use cgmath::{InnerSpace, Vector2};
use ruin_bitmaps::masks_overlap_layers;

use ruin_ecs::physics_2d::{
    Body2D, BodyType2D, CollisionPair, CollisionResolver, NormalizeZero, Vector2D, AABB,
};

bitflags! {
    #[derive(Clone, Copy)]
    struct AnchorDir: u8 {
        const POS_X = 0b0001;
        const NEG_X = 0b0010;
        const POS_Y = 0b0100;
        const NEG_Y = 0b1000;
    }
}

pub struct SimpleCollideAndSlideCollisionResolver {
    slop: f32,
    iterations: u32,
    bias: f32, // fraction of MTV to apply per iteration (0-1, <1 prevents jitter)
}

impl SimpleCollideAndSlideCollisionResolver {
    pub fn new(slop: f32) -> Self {
        Self {
            slop,
            // Enough for dense stacks; walls win each pass so we don't need
            // a huge iteration budget for practical solidity.
            iterations: 4,
            bias: 1.0,
        }
    }

    fn apply_anchor(anchored: &mut AnchorDir, mtv: Vector2D, invert: bool) {
        let x = if invert { -mtv.x } else { mtv.x };
        let y = if invert { -mtv.y } else { mtv.y };
        if x > 0.0 {
            *anchored |= AnchorDir::NEG_X;
        }
        if x < 0.0 {
            *anchored |= AnchorDir::POS_X;
        }
        if y > 0.0 {
            *anchored |= AnchorDir::NEG_Y;
        }
        if y < 0.0 {
            *anchored |= AnchorDir::POS_Y;
        }
    }

    fn axis_blocked(anchored: AnchorDir, component: f32, positive: AnchorDir, negative: AnchorDir) -> bool {
        (component > 0.0 && anchored.contains(positive))
            || (component < 0.0 && anchored.contains(negative))
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

        let mut a_moved = false;
        let mut b_moved = false;

        for a_aabb in a.aabbs.clone() {
            for b_aabb in b.aabbs.clone() {
                let a_hits_b = masks_overlap_layers(a_aabb.masks, b_aabb.layers);
                let b_hits_a = masks_overlap_layers(b_aabb.masks, a_aabb.layers);
                if !a_hits_b && !b_hits_a {
                    continue;
                }
                if let Some(overlap) = compute_mtv(&a_aabb.aabb, &b_aabb.aabb) {
                    let penetration = overlap.magnitude();
                    if penetration <= self.slop {
                        continue;
                    }

                    let normal = overlap.normalize_to_zero();
                    let mtv = normal * penetration * self.bias;

                    match (a.body_type(), b.body_type()) {
                        (BodyType2D::Rigid, BodyType2D::Static) => {
                            a.position -= mtv;
                            a_moved = true;
                            Self::apply_anchor(&mut anchored[a_idx], mtv, false);

                            // Clamp velocity along anchored axes so bodies
                            // don't keep driving into walls frame after frame.
                            let flags = anchored[a_idx];
                            if flags.contains(AnchorDir::POS_X) && a.velocity.x < 0.0 {
                                a.velocity.x = 0.0;
                            }
                            if flags.contains(AnchorDir::NEG_X) && a.velocity.x > 0.0 {
                                a.velocity.x = 0.0;
                            }
                            if flags.contains(AnchorDir::POS_Y) && a.velocity.y < 0.0 {
                                a.velocity.y = 0.0;
                            }
                            if flags.contains(AnchorDir::NEG_Y) && a.velocity.y > 0.0 {
                                a.velocity.y = 0.0;
                            }
                        }
                        (BodyType2D::Static, BodyType2D::Rigid) => {
                            b.position += mtv;
                            b_moved = true;
                            Self::apply_anchor(&mut anchored[b_idx], mtv, true);

                            let flags = anchored[b_idx];
                            if flags.contains(AnchorDir::POS_X) && b.velocity.x < 0.0 {
                                b.velocity.x = 0.0;
                            }
                            if flags.contains(AnchorDir::NEG_X) && b.velocity.x > 0.0 {
                                b.velocity.x = 0.0;
                            }
                            if flags.contains(AnchorDir::POS_Y) && b.velocity.y < 0.0 {
                                b.velocity.y = 0.0;
                            }
                            if flags.contains(AnchorDir::NEG_Y) && b.velocity.y > 0.0 {
                                b.velocity.y = 0.0;
                            }
                        }
                        (BodyType2D::Rigid, BodyType2D::Rigid) => {
                            // Split MTV, then transfer any blocked half onto the free body
                            // so wall-anchored stacks still fully separate.
                            let mut move_a = mtv * 0.5;
                            let mut move_b = mtv * 0.5;

                            // Anchor flags use the same MTV-sign convention as the
                            // original resolver for both bodies in the pair.
                            let a_block_x = Self::axis_blocked(
                                anchored[a_idx],
                                mtv.x,
                                AnchorDir::POS_X,
                                AnchorDir::NEG_X,
                            );
                            let a_block_y = Self::axis_blocked(
                                anchored[a_idx],
                                mtv.y,
                                AnchorDir::POS_Y,
                                AnchorDir::NEG_Y,
                            );
                            let b_block_x = Self::axis_blocked(
                                anchored[b_idx],
                                mtv.x,
                                AnchorDir::POS_X,
                                AnchorDir::NEG_X,
                            );
                            let b_block_y = Self::axis_blocked(
                                anchored[b_idx],
                                mtv.y,
                                AnchorDir::POS_Y,
                                AnchorDir::NEG_Y,
                            );

                            if a_block_x {
                                move_a.x = 0.0;
                                move_b.x = if b_block_x { 0.0 } else { mtv.x };
                            } else if b_block_x {
                                move_b.x = 0.0;
                                move_a.x = mtv.x;
                            }

                            if a_block_y {
                                move_a.y = 0.0;
                                move_b.y = if b_block_y { 0.0 } else { mtv.y };
                            } else if b_block_y {
                                move_b.y = 0.0;
                                move_a.y = mtv.y;
                            }

                            if move_a.x != 0.0 || move_a.y != 0.0 {
                                a.position -= move_a;
                                a_moved = true;
                            }
                            if move_b.x != 0.0 || move_b.y != 0.0 {
                                b.position += move_b;
                                b_moved = true;
                            }

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

        // Keep AABBs in sync so later pairs / iterations see corrected bounds.
        // Without this, multi-pass resolution re-applies stale overlaps and
        // dense crowds get shoved through terrain.
        if a_moved {
            a.sync_aabbs();
        }
        if b_moved {
            b.sync_aabbs();
        }
    }

    fn separate_from_all_statics(
        &self,
        bodies: &mut Vec<Body2D>,
        body_idx: usize,
        static_indices: &[usize],
        anchored: &mut [AnchorDir],
    ) {
        if !matches!(bodies[body_idx].body_type(), BodyType2D::Rigid) {
            return;
        }
        for &static_idx in static_indices {
            self.resolve_pair(
                bodies,
                &CollisionPair {
                    a: body_idx,
                    b: static_idx,
                },
                anchored,
            );
        }
    }
}

impl CollisionResolver for SimpleCollideAndSlideCollisionResolver {
    fn resolve(&mut self, bodies: &mut Vec<Body2D>, collisions: &Vec<CollisionPair>) {
        if bodies.is_empty() || collisions.is_empty() {
            return;
        }

        let mut static_pairs = Vec::new();
        let mut dynamic_pairs = Vec::new();
        let mut static_indices = Vec::new();

        for (i, body) in bodies.iter().enumerate() {
            if matches!(body.body_type(), BodyType2D::Static) {
                static_indices.push(i);
            }
        }

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

        let mut anchored = vec![AnchorDir::empty(); bodies.len()];

        // Seed anchors from current terrain overlaps.
        for pair in &static_pairs {
            self.resolve_pair(bodies, pair, &mut anchored);
        }

        for _ in 0..self.iterations {
            for pair in &dynamic_pairs {
                let (a, b) = (pair.a, pair.b);
                let before_a = bodies[a].position;
                let before_b = bodies[b].position;
                self.resolve_pair(bodies, pair, &mut anchored);
                if bodies[a].position != before_a {
                    self.separate_from_all_statics(bodies, a, &static_indices, &mut anchored);
                }
                if bodies[b].position != before_b {
                    self.separate_from_all_statics(bodies, b, &static_indices, &mut anchored);
                }
            }
            for pair in &static_pairs {
                self.resolve_pair(bodies, pair, &mut anchored);
            }
        }

        // Final sweep: every rigid vs every static.
        for i in 0..bodies.len() {
            if matches!(bodies[i].body_type(), BodyType2D::Rigid) {
                self.separate_from_all_statics(bodies, i, &static_indices, &mut anchored);
            }
        }
    }
}

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
