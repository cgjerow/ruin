use std::time::Instant;

use ruin_bitmaps::masks_overlap_layers;
use ruin_bvh::BVH;
use ruin_ecs::physics_2d::{
    body_in_range, AABB, Body2D, BodyType2D, CollisionDetector, CollisionPair, Point2D,
};

pub struct BvhCollisionDetector {
    player_position: Point2D,
}

impl BvhCollisionDetector {
    pub fn new() -> BvhCollisionDetector {
        BvhCollisionDetector {
            player_position: Point2D { x: 0.0, y: 0.0 },
        }
    }
}

impl CollisionDetector for BvhCollisionDetector {
    fn update_player_position(&mut self, position: Point2D, _physics_range: f32) {
        self.player_position = position;
    }

    fn broad_phase(&mut self, bodies: &Vec<Body2D>, center: Point2D, range: f32) -> Vec<CollisionPair> {
        // TODO: Implement BVH-based broad phase query.
        // Currently builds a BVH but never queries it — falls back to O(n²) brute force.
        // BVH build commented out until query logic is implemented.
        /*
        let mut bvh = BVH::build(
            &mut bodies
                .iter()
                .enumerate()
                .map(|(i, b)| (b.aabb_superset.clone(), i))
                .collect::<Vec<(AABB, usize)>>(),
            4,
        );
        //println!("Inserts {:?}", i.elapsed().as_secs_f64());
        let _i = Instant::now();
        */

        let mut pairs = Vec::new();

        // TODO: Query BVH for overlapping pairs within range
        // For now, fall back to grid-style tier filtering on all bodies
        /*
        let body_count = bodies.len();
        for i in 0..body_count {
            if bodies[i].colliders.is_empty() {
                continue;
            }
            let a_in_range = body_in_range(&bodies[i], center, range);
            for j in (i + 1)..body_count {
                if bodies[j].colliders.is_empty() {
                    continue;
                }
                let b_in_range = body_in_range(&bodies[j], center, range);

                // Tier 3: both out-of-range, entity-entity → skip
                if !a_in_range && !b_in_range {
                    if !matches!(bodies[i].body_type(), BodyType2D::Static)
                        && !matches!(bodies[j].body_type(), BodyType2D::Static)
                    {
                        continue;
                    }
                }

                if bodies[i].aabb_superset.overlaps(&bodies[j].aabb_superset)
                    && (masks_overlap_layers(
                        bodies[i].masks_superset(),
                        bodies[j].layers_superset(),
                    ) || masks_overlap_layers(
                        bodies[j].masks_superset(),
                        bodies[i].layers_superset(),
                    ))
                {
                    pairs.push(CollisionPair { a: i, b: j });
                }
            }
        }
        */

        pairs
    }

    fn narrow_phase(&mut self, broad_phase_results: &Vec<CollisionPair>) -> Vec<CollisionPair> {
        return broad_phase_results.clone();
    }
}
