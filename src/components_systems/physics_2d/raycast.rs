use cgmath::Vector2;

use crate::components_systems::Entity;

#[derive(Debug, Clone)]
pub struct RayCast2D {
    pub id: Entity,
    pub offset: Vector2<f32>,
    pub direction_normal: Vector2<f32>,
    pub magnitude: f32,
    pub masks: u8,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct RayCastHit2D {
    pub origin_parent: Entity,
    pub origin: Entity,
    pub entity: Entity,
    pub point: Vector2<f32>,
    pub normal: Vector2<f32>, // from a->b
    pub distance: f32,
    pub hit_from_inside: bool,
    pub hit_fraction: f32,
}

pub fn ray_vs_aabb(
    ray_origin: Vector2<f32>,
    ray_dir: Vector2<f32>, // must be normalized
    max_distance: f32,
    aabb_center: Vector2<f32>,
    aabb_half_extents: Vector2<f32>,
    origin_entity: Entity, // the ray's entity
    hit_entity: Entity,    // the aabb's entity
) -> Option<RayCastHit2D> {
    let inv_dir = Vector2::new(1.0 / ray_dir.x, 1.0 / ray_dir.y);

    // Calculate slab intersections for X and Y
    let t1 = (aabb_center.x - aabb_half_extents.x - ray_origin.x) * inv_dir.x;
    let t2 = (aabb_center.x + aabb_half_extents.x - ray_origin.x) * inv_dir.x;
    let t3 = (aabb_center.y - aabb_half_extents.y - ray_origin.y) * inv_dir.y;
    let t4 = (aabb_center.y + aabb_half_extents.y - ray_origin.y) * inv_dir.y;

    // Find the min and max distances along the ray for the slabs
    let tmin = t1.min(t2).max(t3.min(t4));
    let tmax = t1.max(t2).min(t3.max(t4));

    // If tmax < 0, the AABB is behind the ray
    if tmax < 0.0 {
        return None;
    }

    // If tmin > tmax, ray misses the AABB
    if tmin > tmax {
        return None;
    }

    // The ray hits at tmin unless inside the box
    let t_hit = if tmin < 0.0 { 0.0 } else { tmin };

    if t_hit > max_distance {
        return None; // beyond ray max distance
    }

    // Calculate hit point
    let hit_point = ray_origin + ray_dir * t_hit;

    // Determine which face was hit to get normal
    // For each axis, check if hit_point is near the min or max face of AABB

    let mut normal = Vector2::new(0.0, 0.0);
    let epsilon = 1e-6;

    if (hit_point.x - (aabb_center.x - aabb_half_extents.x)).abs() < epsilon {
        normal = Vector2::new(-1.0, 0.0);
    } else if (hit_point.x - (aabb_center.x + aabb_half_extents.x)).abs() < epsilon {
        normal = Vector2::new(1.0, 0.0);
    } else if (hit_point.y - (aabb_center.y - aabb_half_extents.y)).abs() < epsilon {
        normal = Vector2::new(0.0, -1.0);
    } else if (hit_point.y - (aabb_center.y + aabb_half_extents.y)).abs() < epsilon {
        normal = Vector2::new(0.0, 1.0);
    }

    // Detect if ray started inside AABB
    let hit_from_inside = tmin < 0.0;

    Some(RayCastHit2D {
        origin_parent: origin_entity, // or pass separately if needed
        origin: origin_entity,
        entity: hit_entity,
        point: hit_point,
        normal,
        distance: t_hit,
        hit_from_inside,
        hit_fraction: t_hit / max_distance,
    })
}
