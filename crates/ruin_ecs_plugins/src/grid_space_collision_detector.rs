use std::collections::HashMap;

use ruin_bitmaps::masks_overlap_layers;
use ruin_ecs::physics_2d::{
    body_in_range, is_terrain, Body2D, BodyType2D, CollisionDetector, CollisionPair, Index,
    Point2D, Unit,
};

type GridCoord = (i32, i32);

#[derive(Debug)]
struct SpatialGrid {
    dynamic_tiles: HashMap<GridCoord, Vec<Index>>, // body indices
    static_tiles: HashMap<GridCoord, Vec<Index>>,
    tile_size: Unit,
    grid_radius: i32,
}

impl SpatialGrid {
    pub fn new(tile_size: Unit, radius: i32) -> SpatialGrid {
        SpatialGrid {
            dynamic_tiles: HashMap::new(),
            static_tiles: HashMap::new(),
            tile_size,
            grid_radius: radius,
        }
    }
}

pub struct GridSpaceCollisionDetector {
    grid: SpatialGrid,
    player_position: Point2D,
}

impl GridSpaceCollisionDetector {
    pub fn new(tile_size: Unit, grid_radius: i32) -> GridSpaceCollisionDetector {
        GridSpaceCollisionDetector {
            grid: SpatialGrid::new(tile_size, grid_radius),
            player_position: Point2D { x: 0.0, y: 0.0 },
        }
    }

    fn insert_body_into_grid(
        grid: &mut HashMap<GridCoord, Vec<Index>>,
        body: &Body2D,
        body_index: Index,
        tile_size: Unit,
    ) {
        let aabb = &body.aabb_superset;
        let min = aabb.min;
        let max = aabb.max;

        let min_tile_x = (min.x / tile_size).floor() as i32;
        let min_tile_y = (min.y / tile_size).floor() as i32;
        let max_tile_x = (max.x / tile_size).floor() as i32;
        let max_tile_y = (max.y / tile_size).floor() as i32;

        for x in min_tile_x..=max_tile_x {
            for y in min_tile_y..=max_tile_y {
                grid.entry((x, y)).or_default().push(body_index);
            }
        }
        /*
        let tiles_covered = (max_tile_x - min_tile_x + 1) * (max_tile_y - min_tile_y + 1);
        if tiles_covered > 9 {
            println!("Body {} touches {} tiles", body_index, tiles_covered);
        }
        */
    }
}

impl CollisionDetector for GridSpaceCollisionDetector {
    fn update_player_position(&mut self, position: Point2D, _physics_range: f32) {
        self.player_position = position;
    }

    fn broad_phase(&mut self, bodies: &Vec<Body2D>, center: Point2D, range: f32) -> Vec<CollisionPair> {
        self.grid.dynamic_tiles.clear();
        self.grid.static_tiles.clear();

        // Insert all collidable bodies. Entity-entity pairs for out-of-range
        // bodies are skipped during pair generation (Tier 3); entity-terrain
        // pairs are always generated so walls stay solid everywhere.
        let mut bodies_inserted = 0;
        let mut bodies_out_of_range = 0;
        let range_sq = (range + self.grid.tile_size) * (range + self.grid.tile_size);
        for (i, body) in bodies.iter().enumerate() {
            if body.colliders.is_empty() {
                continue;
            }

            let dx = body.position.x - center.x;
            let dy = body.position.y - center.y;
            if !is_terrain(body) && dx * dx + dy * dy > range_sq {
                bodies_out_of_range += 1;
            }
            bodies_inserted += 1;

            let target_map =
                if matches!(body.body_type(), BodyType2D::Rigid | BodyType2D::Kinematic) {
                    &mut self.grid.dynamic_tiles
                } else {
                    &mut self.grid.static_tiles
                };

            Self::insert_body_into_grid(target_map, body, i, self.grid.tile_size);
        }
        let mut pairs = Vec::new();
        static EMPTY_VEC: Vec<usize> = Vec::new();
        let mut pairs_checked = 0;
        let mut pairs_added = 0;
        let mut visited_skipped = 0;
        let mut visited = std::collections::HashSet::new();

        let tile_size = self.grid.tile_size;
        let center_tile_x = (center.x / tile_size).floor() as i32;
        let center_tile_y = (center.y / tile_size).floor() as i32;
        let range_tiles = (range / tile_size).ceil() as i32;

        // Entity-terrain everywhere; entity-entity only near the player.
        for (&tile, dynamic) in &self.grid.dynamic_tiles {
            let static_ = self.grid.static_tiles.get(&tile).unwrap_or(&EMPTY_VEC);
            let tile_in_range = (tile.0 - center_tile_x).abs() <= range_tiles
                && (tile.1 - center_tile_y).abs() <= range_tiles;

            if tile_in_range {
                for i in 0..dynamic.len() {
                    for j in (i + 1)..dynamic.len() {
                        pairs_checked += 1;
                        let a = dynamic[i];
                        let b = dynamic[j];
                        if visited.insert((a.min(b), a.max(b))) {
                            let a_in_range = body_in_range(&bodies[a], center, range);
                            let b_in_range = body_in_range(&bodies[b], center, range);
                            if !a_in_range && !b_in_range {
                                continue;
                            }

                            if (masks_overlap_layers(
                                bodies[a].masks_superset(),
                                bodies[b].layers_superset(),
                            ) || masks_overlap_layers(
                                bodies[b].masks_superset(),
                                bodies[a].layers_superset(),
                            )) && bodies[a].aabb_superset.overlaps(&bodies[b].aabb_superset)
                            {
                                pairs.push(CollisionPair { a, b });
                                pairs_added += 1;
                            }
                        } else {
                            visited_skipped += 1;
                        }
                    }
                }
            }

            // Dynamic vs static — always (terrain stays solid off-screen)
            for &a in dynamic {
                for &b in static_ {
                    pairs_checked += 1;
                    if visited.insert((a.min(b), a.max(b))) {
                        if bodies[a].aabb_superset.overlaps(&bodies[b].aabb_superset) {
                            pairs.push(CollisionPair { a, b });
                            pairs_added += 1;
                        }
                    } else {
                        visited_skipped += 1;
                    }
                }
            }
        }
        pairs
    }

    fn narrow_phase(&mut self, broad_phase_results: &Vec<CollisionPair>) -> Vec<CollisionPair> {
        return broad_phase_results.clone();
    }
}
