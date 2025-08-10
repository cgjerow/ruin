use std::{collections::HashMap, time::Instant};

use cgmath::{InnerSpace, Vector2};

use crate::Entity;

pub type Index = usize;
pub type Unit = f32;
pub type TimeUnit = f32;
pub type Point2D = Vector2<Unit>;
pub type Vector2D = Vector2<Unit>;
pub type HalfExtents = Vector2<Unit>;
pub type MaskLayerBitmap = u8;
pub type PositionedShape = (Shape2D, Point2D);
pub type OffsetShape = (Shape2D, Point2D);

trait NormalizeZero {
    fn normalize_to_zero(self) -> Self;
}

impl NormalizeZero for Vector2D {
    fn normalize_to_zero(self) -> Self {
        if self.magnitude2() > std::f32::EPSILON {
            self.normalize()
        } else {
            Vector2D::new(0.0, 0.0)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Shape2D {
    Circle { radius: Unit },
    Rectangle { half_extents: HalfExtents },
}

impl Shape2D {
    pub fn compute_aabb(&self, center: Point2D) -> AABB {
        match self {
            Shape2D::Circle { radius } => {
                let r = *radius;
                AABB {
                    min: center - Vector2::new(r, r),
                    max: center + Vector2::new(r, r),
                }
            }
            Shape2D::Rectangle { half_extents } => AABB {
                min: center - *half_extents,
                max: center + *half_extents,
            },
        }
    }

    pub fn half_extents(&self) -> Vector2<f32> {
        match *self {
            Shape2D::Rectangle { half_extents } => half_extents,
            Shape2D::Circle { radius } => Vector2 {
                x: radius,
                y: radius,
            },
        }
    }

    pub fn scale(&self, scale: Vector2<f32>) -> Self {
        match *self {
            Shape2D::Rectangle { half_extents } => Shape2D::Rectangle {
                half_extents: Vector2 {
                    x: half_extents.x * scale.x,
                    y: half_extents.y * scale.y,
                },
            },
            Shape2D::Circle { radius } => Shape2D::Circle {
                radius: radius * scale.magnitude(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Area2D {
    pub shape: Shape2D,
    pub offset: Vector2D,
    pub layers: MaskLayerBitmap,
    pub masks: MaskLayerBitmap,
    pub active: bool,
}

impl Area2D {
    pub fn compute_aabb(&self, body_pos: Vector2<f32>) -> AABB {
        self.shape.compute_aabb(body_pos + self.offset)
    }

    pub fn matches_layer(&self, other: &Area2D) -> bool {
        self.active && other.active && (self.masks & other.layers) != 0
    }
}

pub struct ShapeSystem {}
impl ShapeSystem {
    pub fn superset(aabbs: &Vec<AABBMasksAndLayers>) -> AABB {
        assert!(
            !aabbs.is_empty(),
            "Cannot compute superset of empty AABB list"
        );

        let mut superset = aabbs[0].aabb;
        for aabb in aabbs.iter().skip(1) {
            superset.merge(&aabb.aabb);
        }

        return superset;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AABBMasksAndLayers {
    aabb: AABB,
    masks: MaskLayerBitmap,
    layers: MaskLayerBitmap,
}

#[derive(Debug, Copy, Clone)]
pub struct AABB {
    min: Point2D,
    max: Point2D,
}

impl AABB {
    pub fn merge(&mut self, other: &AABB) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
    }

    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Point2D::new(0.0, 0.0),
            max: Point2D::new(0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyType2D {
    Rigid,
    Static,
    Kinematic,
    Trigger,
}

impl From<u8> for BodyType2D {
    fn from(value: u8) -> Self {
        match value {
            0 => BodyType2D::Rigid,
            1 => BodyType2D::Static,
            2 => BodyType2D::Kinematic,
            3 => BodyType2D::Trigger,
            _ => BodyType2D::Rigid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Body2D {
    pub position: Point2D,
    velocity: Vector2D,
    pub colliders: Vec<Area2D>,
    aabbs: Vec<AABBMasksAndLayers>,
    pub aabb_superset: AABB,
    masks_superset: MaskLayerBitmap,
    layers_superset: MaskLayerBitmap,
    body_type: BodyType2D,
    is_active: bool,
}

impl Body2D {
    pub fn new(
        position: Point2D,
        velocity: Vector2D,
        body_type: BodyType2D,
        is_active: bool,
    ) -> Self {
        Self {
            position,
            velocity,
            body_type,
            is_active,
            colliders: Vec::new(),
            aabbs: Vec::new(),
            aabb_superset: AABB::default(),
            masks_superset: 0,
            layers_superset: 0,
        }
    }

    fn push_collider(&mut self, collider: Area2D) {
        self.masks_superset |= collider.masks;
        self.layers_superset |= collider.layers;
        let world_position = self.position + collider.offset;
        let aabb = collider.shape.compute_aabb(world_position);
        if self.aabbs.len() == 0 {
            self.aabb_superset = aabb.clone();
        } else {
            self.aabb_superset.merge(&aabb);
        }
        self.aabbs.push(AABBMasksAndLayers {
            aabb,
            masks: collider.masks,
            layers: collider.layers,
        });
        self.colliders.push(collider);
    }

    pub fn integrate(&mut self, dt: TimeUnit) {
        if !self.is_active {
            return;
        }

        match self.body_type {
            BodyType2D::Rigid | BodyType2D::Kinematic => {
                self.position += self.velocity * dt;

                self.aabbs.clear();
                for collider in &self.colliders {
                    let center = self.position + collider.offset;
                    let aabb = collider.compute_aabb(center);
                    self.aabbs.push(AABBMasksAndLayers {
                        aabb,
                        masks: collider.masks,
                        layers: collider.layers,
                    });
                }

                if self.aabbs.len() > 0 {
                    self.aabb_superset = ShapeSystem::superset(&self.aabbs);
                }
            }
            _ => {}
        }
    }
}

pub struct PhysicsWorld {
    pub bodies: Vec<Body2D>,
    pub entity_map: HashMap<Entity, usize>,
    grid: SpatialGrid,
    player_pos: Point2D,
    slop: f32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            entity_map: HashMap::new(),
            grid: SpatialGrid {
                dynamic_tiles: HashMap::new(),
                static_tiles: HashMap::new(),
                tile_size: 3.0,
                grid_radius: 100,
            },
            player_pos: Point2D { x: 0.0, y: 0.0 },
            slop: 0.0,
        }
    }

    pub fn get_velocity(&self, entity: &Entity) -> Vector2D {
        self.bodies[*self.entity_map.get(entity).unwrap()].velocity
    }

    pub fn set_velocity(&mut self, entity: &Entity, velocity: Vector2D) {
        if let Some(index) = self.entity_map.get(entity) {
            self.bodies[*index].velocity = velocity;
        }
    }

    pub fn step(&mut self, dt: TimeUnit) {
        let i = Instant::now();
        self.integrate(dt); // Move bodies based on velocity
                            //println!("Integrate {:?}", i.elapsed().as_secs_f64());
        let i = Instant::now();
        let overlaps = self.broad_phase(); // Basic AABB overlap test
                                           //println!("overlaps {:?}", i.elapsed().as_secs_f64());
        let i = Instant::now();
        self.resolve_collisions(&overlaps); // Push back overlapping bodies
                                            //println!("Resolves {:?}", i.elapsed().as_secs_f64());
    }

    fn resolve_collisions(&mut self, pairs: &Vec<CollisionPair>) {
        for pair in pairs {
            let (a_idx, b_idx) = (pair.a, pair.b);
            let (a, b) = {
                let (left, right) = self.bodies.split_at_mut(std::cmp::max(a_idx, b_idx));
                if a_idx < b_idx {
                    (&mut left[a_idx], &mut right[0])
                } else {
                    (&mut right[0], &mut left[b_idx])
                }
            };

            // Skip if both are Kinematic or Trigger
            if matches!(a.body_type, BodyType2D::Kinematic | BodyType2D::Trigger)
                && matches!(b.body_type, BodyType2D::Kinematic | BodyType2D::Trigger)
            {
                continue;
            }

            for a_aabb in &a.aabbs {
                for b_aabb in &b.aabbs {
                    if Self::masks_overlap_layers(a_aabb.masks, b_aabb.layers)
                        && a_aabb.aabb.overlaps(&b_aabb.aabb)
                    {
                        if let Some(overlap) = compute_mtv(&a_aabb.aabb, &b_aabb.aabb) {
                            let penetration = overlap.magnitude();
                            if penetration <= self.slop {
                                continue; // Ignore very small penetrations
                            }

                            let normal = overlap.normalize_to_zero();
                            let mtv = normal * penetration;

                            match (&a.body_type, &b.body_type) {
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

    fn masks_overlap_layers(a: MaskLayerBitmap, b: MaskLayerBitmap) -> bool {
        a & b > 0
    }

    fn integrate(&mut self, dt: TimeUnit) {
        for body in &mut self.bodies {
            body.integrate(dt);
        }
        self.player_pos = self.bodies[0].position.clone();
        //println!("{:?}", self.player_pos);
    }

    pub fn add_collider(&mut self, entity: &Entity, collider: Area2D) {
        if let Some(index) = self.entity_map.get(entity) {
            let body = &mut self.bodies[*index];
            body.push_collider(collider);
        } else {
            eprintln!(
                "Warning: Tried to add a collider to nonexistent body {:?}",
                entity
            );
        }
    }

    fn broad_phase(&mut self) -> Vec<CollisionPair> {
        let i = Instant::now();
        self.grid.dynamic_tiles.clear();
        self.grid.static_tiles.clear();

        for (i, body) in self
            .bodies
            .iter()
            .filter(|b| !b.colliders.is_empty())
            .enumerate()
        {
            let target_map = if matches!(body.body_type, BodyType2D::Rigid | BodyType2D::Kinematic)
            {
                &mut self.grid.dynamic_tiles
            } else {
                &mut self.grid.static_tiles
            };

            Self::insert_body_into_grid(target_map, body, i, self.grid.tile_size);
        }
        //println!("Inserts {:?}", i.elapsed().as_secs_f64());
        let i = Instant::now();

        let mut pairs = Vec::new();
        let tile_size = self.grid.tile_size;
        let center_tile_x = (self.player_pos.x / tile_size).floor() as i32;
        let center_tile_y = (self.player_pos.y / tile_size).floor() as i32;
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
                            if (Self::masks_overlap_layers(
                                self.bodies[a].masks_superset,
                                self.bodies[b].layers_superset,
                            ) || Self::masks_overlap_layers(
                                self.bodies[b].masks_superset,
                                self.bodies[a].layers_superset,
                            )) && self.bodies[a]
                                .aabb_superset
                                .overlaps(&self.bodies[b].aabb_superset)
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
                            if self.bodies[a]
                                .aabb_superset
                                .overlaps(&self.bodies[b].aabb_superset)
                            {
                                pairs.push(CollisionPair { a, b });
                            }
                        }
                    }
                }
            }
        }

        //println!("Visits {:?}", i.elapsed().as_secs_f64());
        pairs
    }

    pub fn add_body(&mut self, entity: Entity, body: Body2D) {
        let index = self.bodies.len();
        self.bodies.push(body);
        self.entity_map.insert(entity, index);
    }

    pub fn positions(&self) -> HashMap<Entity, Point2D> {
        self.entity_map
            .iter()
            .map(|(entity, &index)| (*entity, self.bodies[index].position))
            .collect()
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

#[derive(Debug)]
pub struct CollisionPair {
    pub a: Index,
    pub b: Index,
}

type GridCoord = (i32, i32);

#[derive(Debug)]
struct SpatialGrid {
    dynamic_tiles: HashMap<GridCoord, Vec<Index>>, // body indices
    static_tiles: HashMap<GridCoord, Vec<Index>>,
    tile_size: Unit,
    grid_radius: i32,
}
