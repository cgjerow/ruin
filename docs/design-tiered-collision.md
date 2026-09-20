# Design: Tiered Collision System

> **Status**: ✅ Implemented (as of 2025-07-04). All core changes from this design are in place.
> **Profiling**: 1500 bodies at 60 Hz → ~5.5 ms total physics (85% broad phase). Stable ~120 FPS. See `docs/requirements.md` for details.

## Problem

With 1000 entities, collision detection resolves to O(n²) even with spatial hashing, because the grid radius (100 tiles) covers the entire arena. Full collision for all entities is unsustainable.

## Solution: Tiered Collision by Physics Range

Physics range is a configurable radius (in world units) centered on the player. Entities are classified as **in-range** or **out-of-range** based on whether their AABB overlaps the range circle.

### Tiers

| Tier | In-range bodies | Collision handling |
|------|----------------|-------------------|
| 1 | Both in-range | Full detection + resolution |
| 2 | One in-range, one out | Full detection + resolution |
| 3 | Both out-of-range | Entity-entity SKIPPED; entity-terrain still resolved |

### Key Design Decisions

1. **Integration runs on ALL bodies** — off-screen entities still move via velocity → position, so they're where they should be when the camera approaches
2. **Terrain is always simulated** — static/kinematic bodies (walls, floor) always collide, even for out-of-range entities, so enemies don't clip through walls off-screen
3. **Entity-entity off-screen is fine** — overlapping enemies off-screen is imperceptible; when they come in-range, collision kicks in
4. **Range is independent of camera** — zoom, look-ahead, and rendering don't affect physics

## Changes

### 0. Physics Tick Rate — 300 Hz → 60 Hz

**The single biggest performance win.** Physics at 300 Hz means 5 physics steps per frame at 60 FPS. Reducing to 60 Hz matches the frame rate, cutting physics work by 80%.

```rust
// Before: physics_tick_rate: 1.0 / 300.0
// After:  physics_tick_rate: 1.0 / config.physics_fps  (default 60)
```

Configurable via `physics_fps` in `setup.lua`. Default: **60 Hz**.

### 1. `EngineConfig` — add `physics_range`

```rust
pub struct EngineConfig {
    pub physics_range: f32,  // radius in world units, centered on player
    // ...
}
```

Default: `30.0` (60×60 unit square). Settable via `setup.lua`.

### 2. `PhysicsWorld` — store range + center

```rust
pub struct PhysicsWorld {
    bodies: Vec<Body2D>,
    physics_range: f32,
    player_pos: Point2D,  // center of range circle
    // ...
}

impl PhysicsWorld {
    pub fn step(&mut self, dt: TimeUnit) {
        self.integrate(dt);  // ALL bodies, unchanged
        
        let overlaps = self.collision_detector
            .broad_phase(&self.bodies, self.player_pos, self.physics_range);
        let overlaps = self.collision_detector.narrow_phase(&overlaps);
        self.collision_resolver.resolve(&mut self.bodies, &overlaps);
    }
}
```

### 3. `CollisionDetector` trait — add range parameters

```rust
pub trait CollisionDetector {
    fn update_player_position(&mut self, position: Point2D);
    fn broad_phase(&mut self, bodies: &Vec<Body2D>, center: Point2D, range: f32) -> Vec<CollisionPair>;
    fn narrow_phase(&mut self, broad_phase_results: &Vec<CollisionPair>) -> Vec<CollisionPair>;
}
```

### 4. `GridSpaceCollisionDetector` — tier filtering + range-optimized rebuild

```rust
impl CollisionDetector for GridSpaceCollisionDetector {
    fn broad_phase(&mut self, bodies: &Vec<Body2D>, center: Point2D, range: f32) -> Vec<CollisionPair> {
        // Grid rebuild: only insert bodies within range + 1 tile margin
        let range_sq = (range + tile_size).powi(2);
        for body in bodies.iter() {
            let dist_sq = (body.pos - center).length_squared();
            if dist_sq > range_sq { continue; }  // Skip far-away bodies
            insert_into_grid(body);
        }
        
        // Query only tiles within range
        let range_aabb = AABB::from_center_and_radius(center, range);
        let relevant_tiles = tiles_intersecting(&range_aabb);
        
        let mut pairs = Vec::new();
        for tile in relevant_tiles {
            for (a, b) in tile.pairs() {
                let a_in_range = body_in_range(&bodies[a], center, range);
                let b_in_range = body_in_range(&bodies[b], center, range);
                
                // Tier 3: both out-of-range, entity-entity → skip
                if !a_in_range && !b_in_range {
                    if !is_static_or_kinematic(&bodies[a]) && !is_static_or_kinematic(&bodies[b]) {
                        continue;
                    }
                }
                
                pairs.push(CollisionPair { a, b });
            }
        }
        pairs
    }
}
```

### 5. `BvhCollisionDetector` — same filtering

Mirror the tier logic from the grid detector.

### 6. Lua API — `engine.set_physics_range(radius)`

Runtime-adjustable for effects like fog-of-war or zoom-based culling.

## Performance Impact

With 1500 entities, physics_range = 30.0, player in center of 50×50 arena:

| Metric | Before (300 Hz) | After (60 Hz + tiered) |
|--------|----------------|------------------------|
| Physics steps/frame | 5 (at 60 FPS) | 1 |
| Bodies integrated | 1500 | 1500 (unchanged) |
| Bodies in grid | 1500 | 1500 (unchanged) |
| Tiles queried | ~289 (all) | ~400 (range-based) |
| Pairs generated | ~1.1M (300 Hz) | ~4K (tier 3 filtered) |
| Pairs resolved | ~1.1M (300 Hz) | ~4K |
| Step time | ~20ms (5×4ms) | ~5.5ms |
| Physics budget utilization | 120% (over budget) | 33% (at 120 FPS render) |

### Key Savings Breakdown

1. **300 Hz → 60 Hz**: 80% reduction in physics steps (5→1 per frame)
2. **Tier 3 filtering**: ~90% reduction in collision pairs (skips entity-entity for both-out-of-range)
3. **Combined**: ~95% reduction in total physics work

## Acceptance

- Enemies off-screen still walk on terrain (don't clip through walls)
- Enemies off-screen can overlap each other (no visible effect)
- When camera moves toward off-screen enemies, they start colliding immediately
- `physics_range` configurable via `setup.lua` and runtime via Lua API

## Follow-on: REG-001 Tunneling Fix

The tiered collision system exposed a secondary issue: **dense-crowd terrain tunneling**. When ~200+ entities were pushed against walls (e.g., arena fence), bodies would clip through static colliders despite the tier filtering keeping entity-terrain pairs global.

### Root Causes

1. **Stale AABBs during resolve** — Multi-pass resolution moved bodies but never updated AABBs, so subsequent iterations applied the same overlaps.
2. **Inverted wall velocity clamp** — Velocity was clamped along the MTV direction instead of the separation direction, keeping bodies driving into walls.
3. **Precomputed static pairs only** — Resolution only checked static pairs from broadphase; a body pushed through a wall mid-solve had no new static pair to catch it.
4. **Incomplete MTV transfer** — When anchored against a wall, the free body didn't receive the full MTV, leaving residual stack pressure.

### Fix Summary

- `Body2D::sync_aabbs()` called after every position change in the resolver
- Per-moved-body static re-test (`separate_from_all_statics`) after each dynamic resolution
- Velocity clamped along **separation** direction (out of wall), not MTV direction
- Full MTV transferred to free body when one axis is wall-anchored
- Bidirectional mask/layer checks

See `docs/regressions.md` (REG-001) for full details and regression tests.
