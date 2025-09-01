use std::{collections::HashMap, time::Instant};

use ruin_bitmaps::masks_overlap_layers;
use ruin_ecs::physics_2d::{
    Body2D, BodyType2D, CollisionDetector, CollisionPair, Index, Point2D, Unit, AABB,
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
    fn update_player_position(&mut self, position: Point2D) {
        self.player_position = position;
    }

    fn broad_phase(&mut self, bodies: &Vec<Body2D>) -> Vec<CollisionPair> {
        let _i = Instant::now();
        self.grid.dynamic_tiles.clear();
        self.grid.static_tiles.clear();

        for (i, body) in bodies
            .iter()
            .filter(|b| !b.colliders.is_empty())
            .enumerate()
        {
            let target_map =
                if matches!(body.body_type(), BodyType2D::Rigid | BodyType2D::Kinematic) {
                    &mut self.grid.dynamic_tiles
                } else {
                    &mut self.grid.static_tiles
                };

            Self::insert_body_into_grid(target_map, body, i, self.grid.tile_size);
        }
        //println!("Inserts {:?}", i.elapsed().as_secs_f64());
        let _i = Instant::now();

        let mut pairs = Vec::new();
        let tile_size = self.grid.tile_size;
        let center_tile_x = (self.player_position.x / tile_size).floor() as i32;
        let center_tile_y = (self.player_position.y / tile_size).floor() as i32;
        let radius = self.grid.grid_radius;

        let mut visited = std::collections::HashSet::new();
        static EMPTY_VEC: Vec<usize> = Vec::new();

        // Process dynamic tiles
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

        pairs
    }

    fn narrow_phase(&mut self, broad_phase_results: &Vec<CollisionPair>) -> Vec<CollisionPair> {
        return broad_phase_results.clone();
    }
}
