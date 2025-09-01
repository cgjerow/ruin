use std::{collections::HashMap, time::Instant};

use ruin_bitmaps::masks_overlap_layers;
use ruin_bvh::BVH;
use ruin_ecs::physics_2d::{
    Body2D, BodyType2D, CollisionDetector, CollisionPair, Index, Point2D, Unit, AABB,
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
    fn update_player_position(&mut self, position: Point2D) {
        self.player_position = position;
    }

    fn broad_phase(&mut self, bodies: &Vec<Body2D>) -> Vec<CollisionPair> {
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

        let mut pairs = Vec::new();
        // Process dynamic tiles
        /*
        for (&tile, dynamic) in &self.grid.dynamic_tiles {
            if (tile.0 - center_tile_x).abs() <= radius && (tile.1 - center_tile_y).abs() <= radius
            {
                let static_ = self.grid.static_tiles.get(&tile).unwrap_or(&EMPTY_VEC);

                // Dynamic vs dynamic within the tile
                for i in 0..dynamic.len() {
                    for j in (i + 1)..dynamic.len() {
                        let a = dynamic[i];
                        let b = dynamic[j];
                        if visited.insert((a.min(b), a.max(b))) {
                            if (masks_overlap_layers(
                                bodies[a].masks_superset(),
                                bodies[b].layers_superset(),
                            ) || masks_overlap_layers(
                                bodies[b].masks_superset(),
                                bodies[a].layers_superset(),
                            )) && bodies[a].aabb_superset.overlaps(&bodies[b].aabb_superset)
                            {
                                pairs.push(CollisionPair { a, b });
                            }
                        }
                    }
                }

                // Dynamic vs static within the tile
                for &a in dynamic {
                    for &b in static_ {
                        if visited.insert((a.min(b), a.max(b))) {
                            if bodies[a].aabb_superset.overlaps(&bodies[b].aabb_superset) {
                                pairs.push(CollisionPair { a, b });
                            }
                        }
                    }
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
