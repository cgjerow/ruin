use std::{collections::HashMap, time::Instant};

use cgmath::{InnerSpace, Vector2};
use ruin_bitmaps::MaskLayerBitmap;

use crate::{
    physics_2d::{CollisionDetector, CollisionResolver},
    Entity,
};

pub type Index = usize;
pub type Unit = f32;
pub type TimeUnit = f32;
pub type Point2D = Vector2<Unit>;
pub type Vector2D = Vector2<Unit>;
pub type HalfExtents = Vector2<Unit>;
pub type PositionedShape = (Shape2D, Point2D);
pub type OffsetShape = (Shape2D, Point2D);

pub trait NormalizeZero {
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
    pub aabb: AABB,
    pub masks: MaskLayerBitmap,
    pub layers: MaskLayerBitmap,
}

#[derive(Debug, Copy, Clone)]
pub struct AABB {
    pub min: Point2D,
    pub max: Point2D,
}

impl AABB {
    #[inline]
    pub fn new(min: Point2D, max: Point2D) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn union(self, other: &AABB) -> AABB {
        AABB::new(
            Point2D {
                x: self.min.x.min(other.min.x),
                y: self.min.y.min(other.min.y),
            },
            Point2D {
                x: self.max.x.max(other.max.x),
                y: self.max.y.max(other.max.y),
            },
        )
    }

    #[inline]
    pub fn area(&self) -> Unit {
        let w = (self.max.x - self.min.x).max(0.0);
        let h = (self.max.y - self.min.y).max(0.0);
        w * h
    }

    pub fn merge(&mut self, other: &AABB) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
    }

    #[inline]
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    #[inline]
    pub fn center(&self) -> Point2D {
        Point2D {
            x: (self.min.x + self.max.x) * 0.5,
            y: (self.min.y + self.max.y) * 0.5,
        }
    }

    #[inline]
    pub fn longest_axis(&self) -> usize {
        let extents = Point2D {
            x: self.max.x - self.min.x,
            y: self.max.y - self.min.y,
        };

        if extents.x > extents.y {
            0 // X axis
        } else {
            1 // Y axis
        }
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
    pub velocity: Vector2D,
    pub colliders: Vec<Area2D>,
    pub aabbs: Vec<AABBMasksAndLayers>,
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

    pub fn body_type(&self) -> &BodyType2D {
        return &self.body_type;
    }

    pub fn masks_superset(&self) -> MaskLayerBitmap {
        return self.masks_superset;
    }

    pub fn layers_superset(&self) -> MaskLayerBitmap {
        return self.layers_superset;
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
    collision_detector: Box<dyn CollisionDetector>,
    collision_resolver: Box<dyn CollisionResolver>,
    player_pos: Point2D,
}

impl PhysicsWorld {
    pub fn new(
        collision_detector: Box<dyn CollisionDetector>,
        collision_resolver: Box<dyn CollisionResolver>,
    ) -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
            entity_map: HashMap::new(),
            player_pos: Point2D { x: 0.0, y: 0.0 },
            collision_detector,
            collision_resolver,
        }
    }

    pub fn unload(&mut self) {
        self.bodies.clear();
        self.entity_map.clear();
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
        let _i = Instant::now();
        self.integrate(dt);
        //println!("Integrate {:?}", i.elapsed().as_secs_f64());
        //
        let _i = Instant::now();
        self.collision_detector
            .update_player_position(self.player_pos);
        let overlaps = self.collision_detector.broad_phase(&self.bodies);
        let overlaps = self.collision_detector.narrow_phase(&overlaps);
        //println!("overlaps {:?}", i.elapsed().as_secs_f64());
        //
        let _i = Instant::now();
        self.collision_resolver.resolve(&mut self.bodies, &overlaps);
        //println!("Resolves {:?}", i.elapsed().as_secs_f64());
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
}
