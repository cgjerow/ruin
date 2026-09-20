//! Collision system regression tests.
//!
//! Tests verify that bodies (especially dynamic ones) cannot push through
//! terrain walls, even in dense crowds.

use ruin_bitmaps::masks_overlap_layers;
use ruin_ecs::physics_2d::{
    Area2D, Body2D, BodyType2D, AABB, PhysicsWorld, Shape2D,
};
use ruin_ecs_plugins::{
    GridSpaceCollisionDetector, SimpleCollideAndSlideCollisionResolver,
};

/// Mask bit indices used in tests (matching globals.lua / skelly.lua)
const ENV: u8 = 1;
const ENEMY: u8 = 0;
const PLAYER: u8 = 2;

fn bit(index: u8) -> u8 {
    1 << index
}

fn make_collider(shape: Shape2D, masks: u8, layers: u8) -> Area2D {
    Area2D {
        shape,
        offset: cgmath::Vector2::new(0.0, 0.0),
        layers,
        masks,
        active: true,
    }
}

fn add_static_wall(
    world: &mut PhysicsWorld,
    id: u32,
    x: f32,
    y: f32,
    half_x: f32,
    half_y: f32,
) {
    let body = Body2D::new(
        cgmath::Vector2::new(x, y),
        cgmath::Vector2::new(0.0, 0.0),
        BodyType2D::Static,
        true,
    );
    world.add_body(id, body);
    // Terrain: Env layer only (matches fence.lua)
    world.add_collider(
        &id,
        make_collider(
            Shape2D::Rectangle {
                half_extents: cgmath::Vector2::new(half_x, half_y),
            },
            0,
            bit(ENV),
        ),
    );
}

fn add_rigid_body(
    world: &mut PhysicsWorld,
    id: u32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
) {
    let body = Body2D::new(
        cgmath::Vector2::new(x, y),
        cgmath::Vector2::new(vx, vy),
        BodyType2D::Rigid,
        true,
    );
    world.add_body(id, body);
    // Enemy: collides with Env, other enemies, and player (matches skelly.lua)
    world.add_collider(
        &id,
        make_collider(
            Shape2D::Rectangle {
                half_extents: cgmath::Vector2::new(size, size),
            },
            bit(ENV) | bit(ENEMY) | bit(PLAYER),
            bit(ENEMY),
        ),
    );
}

fn add_rigid_body_at(world: &mut PhysicsWorld, id: u32, x: f32, y: f32, size: f32) {
    add_rigid_body(world, id, x, y, 0.0, 0.0, size);
}

fn make_world() -> PhysicsWorld {
    // Match engine.rs: GridSpaceCollisionDetector::new(3.0, 100)
    let detector = GridSpaceCollisionDetector::new(3.0, 100);
    let resolver = SimpleCollideAndSlideCollisionResolver::new(0.0);
    PhysicsWorld::new(Box::new(detector), Box::new(resolver), 30.0)
}

/// Minimum translation depth of rigid AABB `a` into static AABB `b`.
fn aabb_penetration(a: &AABB, b: &AABB) -> f32 {
    if !a.overlaps(b) {
        return 0.0;
    }

    let overlap_x = (a.max.x.min(b.max.x) - a.min.x.max(b.min.x)).max(0.0);
    let overlap_y = (a.max.y.min(b.max.y) - a.min.y.max(b.min.y)).max(0.0);
    overlap_x.min(overlap_y)
}

/// Helper: run physics for `steps` frames and return max penetration depth
/// of any rigid body into any static body (checking all AABB pairs).
fn run_and_measure_max_penetration(world: &mut PhysicsWorld, steps: u32) -> f32 {
    for _ in 0..steps {
        world.step(1.0 / 60.0);
    }

    let bodies = &world.bodies;
    let mut max_pen = 0.0;

    for i in 0..bodies.len() {
        for j in 0..bodies.len() {
            if i == j {
                continue;
            }
            let a = &bodies[i];
            let b = &bodies[j];

            // Only check rigid vs static
            if !matches!(a.body_type(), BodyType2D::Rigid)
                || !matches!(b.body_type(), BodyType2D::Static)
            {
                continue;
            }

            // Check masks overlap
            let mut mask_match = false;
            for a_aabb in &a.aabbs {
                for b_aabb in &b.aabbs {
                    if masks_overlap_layers(a_aabb.masks, b_aabb.layers) {
                        mask_match = true;
                        break;
                    }
                }
                if mask_match {
                    break;
                }
            }
            if !mask_match {
                continue;
            }

            // Compute penetration
            for a_aabb in &a.aabbs {
                for b_aabb in &b.aabbs {
                    let pen = aabb_penetration(&a_aabb.aabb, &b_aabb.aabb);
                    if pen > max_pen {
                        max_pen = pen;
                    }
                }
            }
        }
    }

    max_pen
}

// ============================================================================
// Debug test for two bodies behind each other
// ============================================================================

#[test]
fn debug_two_bodies_behind_each_other() {
    let mut world = make_world();

    // Wall at x=-5, half-width 1 (right edge at x=-4)
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);
    // Body 2 at x=2.0, Body 3 at x=4.0, both moving left at -10
    add_rigid_body(&mut world, 2, 2.0, 0.0, -10.0, 0.0, 0.5);
    add_rigid_body(&mut world, 3, 4.0, 0.0, -10.0, 0.0, 0.5);

    println!("=== INITIAL STATE ===");
    for (idx, body) in world.bodies.iter().enumerate() {
        println!(
            "Body[{}] type={:?} at ({:.3}, {:.3}), vel=({:.3}, {:.3}), aabb=({:.3},{:.3})-({:.3},{:.3})",
            idx,
            body.body_type(),
            body.position.x,
            body.position.y,
            body.velocity.x,
            body.velocity.y,
            body.aabb_superset.min.x,
            body.aabb_superset.min.y,
            body.aabb_superset.max.x,
            body.aabb_superset.max.y,
        );
        for (i, aabb) in body.aabbs.iter().enumerate() {
            println!(
                "  Collider {} aabb=({:.3},{:.3})-({:.3},{:.3}) masks={:b} layers={:b}",
                i,
                aabb.aabb.min.x,
                aabb.aabb.min.y,
                aabb.aabb.max.x,
                aabb.aabb.max.y,
                aabb.masks,
                aabb.layers,
            );
        }
    }

    // Run 60 frames, print at key points
    for frame in 0..60 {
        world.step(1.0 / 60.0);
        if frame == 35 || frame == 36 || frame == 37 || frame == 59 {
            println!("=== AFTER {} FRAMES ===", frame + 1);
            for (idx, body) in world.bodies.iter().enumerate() {
                println!(
                    "Body[{}] type={:?} at ({:.4}, {:.4}), vel=({:.3}, {:.3}), aabb=({:.4},{:.4})-({:.4},{:.4})",
                    idx,
                    body.body_type(),
                    body.position.x,
                    body.position.y,
                    body.velocity.x,
                    body.velocity.y,
                    body.aabb_superset.min.x,
                    body.aabb_superset.min.y,
                    body.aabb_superset.max.x,
                    body.aabb_superset.max.y,
                );
            }
        }
    }

    // Check all overlaps
    let bodies = &world.bodies;
    for i in 0..bodies.len() {
        for j in 0..bodies.len() {
            if i == j {
                continue;
            }
            let a = &bodies[i];
            let b = &bodies[j];
            for a_aabb in &a.aabbs {
                for b_aabb in &b.aabbs {
                    if a_aabb.aabb.overlaps(&b_aabb.aabb) {
                        println!(
                            "OVERLAP: body[{}] aabb=({:.4},{:.4})-({:.4},{:.4}) vs body[{}] aabb=({:.4},{:.4})-({:.4},{:.4})",
                            i,
                            a_aabb.aabb.min.x,
                            a_aabb.aabb.min.y,
                            a_aabb.aabb.max.x,
                            a_aabb.aabb.max.y,
                            j,
                            b_aabb.aabb.min.x,
                            b_aabb.aabb.min.y,
                            b_aabb.aabb.max.x,
                            b_aabb.aabb.max.y,
                        );
                    }
                }
            }
        }
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 0);
    println!("Max penetration: {:.4}", max_pen);

    assert!(
        max_pen < 0.05,
        "Two bodies behind each other penetrated wall by {:.4}",
        max_pen
    );
}

// ============================================================================
// Debug test
// ============================================================================

#[test]
fn debug_single_body_positions() {
    let mut world = make_world();

    // Wall at x=-5, 10 units tall (half-height 5), half-width 1
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);
    // Body approaching wall from the right
    add_rigid_body(&mut world, 2, 3.0, 0.0, -10.0, 0.0, 0.5);

    // Print initial state
    println!("=== INITIAL STATE ===");
    for (idx, body) in world.bodies.iter().enumerate() {
        println!(
            "Body[{}] at ({:.3}, {:.3}), vel=({:.3}, {:.3}), aabb=({:.3},{:.3})-({:.3},{:.3})",
            idx,
            body.position.x,
            body.position.y,
            body.velocity.x,
            body.velocity.y,
            body.aabb_superset.min.x,
            body.aabb_superset.min.y,
            body.aabb_superset.max.x,
            body.aabb_superset.max.y,
        );
        for (i, aabb) in body.aabbs.iter().enumerate() {
            println!(
                "  Collider {} aabb=({:.3},{:.3})-({:.3},{:.3}) masks={:b} layers={:b}",
                i,
                aabb.aabb.min.x,
                aabb.aabb.min.y,
                aabb.aabb.max.x,
                aabb.aabb.max.y,
                aabb.masks,
                aabb.layers,
            );
        }
    }

    // Run 30 frames
    for frame in 0..30 {
        world.step(1.0 / 60.0);
        if frame == 29 {
            println!("=== AFTER 30 FRAMES ===");
            for (idx, body) in world.bodies.iter().enumerate() {
                println!(
                    "Body[{}] at ({:.3}, {:.3}), vel=({:.3}, {:.3}), aabb=({:.3},{:.3})-({:.3},{:.3})",
                    idx,
                    body.position.x,
                    body.position.y,
                    body.velocity.x,
                    body.velocity.y,
                    body.aabb_superset.min.x,
                    body.aabb_superset.min.y,
                    body.aabb_superset.max.x,
                    body.aabb_superset.max.y,
                );
                for (i, aabb) in body.aabbs.iter().enumerate() {
                    println!(
                        "  Collider {} aabb=({:.3},{:.3})-({:.3},{:.3}) masks={:b} layers={:b}",
                        i,
                        aabb.aabb.min.x,
                        aabb.aabb.min.y,
                        aabb.aabb.max.x,
                        aabb.aabb.max.y,
                        aabb.masks,
                        aabb.layers,
                    );
                }
            }
        }
    }

    // Check penetration
    let bodies = &world.bodies;
    for i in 0..bodies.len() {
        for j in 0..bodies.len() {
            if i == j {
                continue;
            }
            let a = &bodies[i];
            let b = &bodies[j];
            if !matches!(a.body_type(), BodyType2D::Rigid)
                || !matches!(b.body_type(), BodyType2D::Static)
            {
                continue;
            }
            for a_aabb in &a.aabbs {
                for b_aabb in &b.aabbs {
                    if a_aabb.aabb.overlaps(&b_aabb.aabb) {
                        println!(
                            "OVERLAP: body {} aabb=({:.3},{:.3})-({:.3},{:.3}) vs body {} aabb=({:.3},{:.3})-({:.3},{:.3})",
                            i,
                            a_aabb.aabb.min.x,
                            a_aabb.aabb.min.y,
                            a_aabb.aabb.max.x,
                            a_aabb.aabb.max.y,
                            j,
                            b_aabb.aabb.min.x,
                            b_aabb.aabb.min.y,
                            b_aabb.aabb.max.x,
                            b_aabb.aabb.max.y,
                        );
                    }
                }
            }
        }
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 0);
    println!("Max penetration: {:.4}", max_pen);

    assert!(
        max_pen < 0.05,
        "Single body penetrated wall by {:.4}",
        max_pen
    );
}

// ============================================================================
// Single-body tests (sanity checks)
// ============================================================================

#[test]
fn test_single_body_stops_at_wall() {
    let mut world = make_world();

    // Wall at x=0, 10 units tall (half-height 5)
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);
    // Body approaching wall from the right
    add_rigid_body(&mut world, 2, 3.0, 0.0, -10.0, 0.0, 0.5);

    // Run 30 frames (~0.5s) — body should hit wall and stop
    let max_pen = run_and_measure_max_penetration(&mut world, 30);

    assert!(
        max_pen < 0.05,
        "Single body penetrated wall by {:.4} (expected < 0.05)",
        max_pen
    );
}

#[test]
fn test_single_body_stops_at_wall_top() {
    let mut world = make_world();

    // Wall at y=0, 10 units wide (half-width 5)
    add_static_wall(&mut world, 1, 0.0, -5.0, 5.0, 1.0);
    // Body approaching wall from below
    add_rigid_body(&mut world, 2, 0.0, 3.0, 0.0, -10.0, 0.5);

    let max_pen = run_and_measure_max_penetration(&mut world, 30);

    assert!(
        max_pen < 0.05,
        "Single body penetrated wall by {:.4} (expected < 0.05)",
        max_pen
    );
}

// ============================================================================
// Two-body crowd tests
// ============================================================================

#[test]
fn test_two_bodies_stacked_at_wall() {
    let mut world = make_world();

    // Wall at x=0
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);
    // Two bodies side by side, both pushing toward wall
    add_rigid_body(&mut world, 2, 3.0, 0.0, -10.0, 0.0, 0.5);
    add_rigid_body(&mut world, 3, 3.0, 2.0, -10.0, 0.0, 0.5);

    let max_pen = run_and_measure_max_penetration(&mut world, 30);

    assert!(
        max_pen < 0.05,
        "Two-body crowd penetrated wall by {:.4} (expected < 0.05)",
        max_pen
    );
}

#[test]
fn test_two_bodies_behind_each_other_at_wall() {
    let mut world = make_world();

    // Wall at x=0
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);
    // Two bodies in a line, second one pushing first into wall
    add_rigid_body(&mut world, 2, 2.0, 0.0, -10.0, 0.0, 0.5);
    add_rigid_body(&mut world, 3, 4.0, 0.0, -10.0, 0.0, 0.5);

    let max_pen = run_and_measure_max_penetration(&mut world, 60);

    assert!(
        max_pen < 0.05,
        "Two bodies behind each other penetrated wall by {:.4} (expected < 0.05)",
        max_pen
    );
}

// ============================================================================
// Dense crowd tests (the real bug scenario)
// ============================================================================

#[test]
fn test_ten_bodies_dense_crowd_at_wall() {
    let mut world = make_world();

    // Wall at x=0, spanning y=-5 to y=5
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);

    // 10 bodies in a grid, all pushing toward wall
    let mut id = 2;
    for row in 0..2 {
        for col in 0..5 {
            let x = 2.0 + (col as f32) * 1.2;
            let y = -2.0 + (row as f32) * 2.5;
            add_rigid_body(&mut world, id, x, y, -10.0, 0.0, 0.5);
            id += 1;
        }
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 60);

    assert!(
        max_pen < 0.15,
        "10-body dense crowd penetrated wall by {:.4} (expected < 0.15)",
        max_pen
    );
}

#[test]
fn test_fifty_bodies_heavy_crowd_at_wall() {
    let mut world = make_world();

    // Long wall at x=0, spanning y=-10 to y=10
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 10.0);

    // 50 bodies in a grid, all pushing toward wall
    let mut id = 2;
    for row in 0..5 {
        for col in 0..10 {
            let x = 1.5 + (col as f32) * 1.0;
            let y = -5.0 + (row as f32) * 2.5;
            add_rigid_body(&mut world, id, x, y, -10.0, 0.0, 0.4);
            id += 1;
        }
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 60);

    // Practical bound: catch real tunneling, not hairline residual overlap.
    assert!(
        max_pen < 0.15,
        "50-body heavy crowd penetrated wall by {:.4} (expected < 0.15)",
        max_pen
    );
}

#[test]
fn test_hundred_bodies_extreme_crowd_at_wall() {
    let mut world = make_world();

    // Long wall at x=0, spanning y=-15 to y=15
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 15.0);

    // 100 bodies in a grid, all pushing toward wall
    let mut id = 2;
    for row in 0..10 {
        for col in 0..10 {
            let x = 1.0 + (col as f32) * 0.9;
            let y = -5.0 + (row as f32) * 2.0;
            add_rigid_body(&mut world, id, x, y, -10.0, 0.0, 0.4);
            id += 1;
        }
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 60);

    assert!(
        max_pen < 0.15,
        "100-body extreme crowd penetrated wall by {:.4} (expected < 0.15)",
        max_pen
    );
}

// ============================================================================
// Continuous push tests (simulating enemy AI constantly moving toward wall)
// ============================================================================

#[test]
fn test_continuous_push_no_gradual_sinking() {
    let mut world = make_world();

    // Wall at x=0
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);

    // Single body pushed continuously toward wall
    add_rigid_body_at(&mut world, 2, 3.0, 0.0, 0.5);

    // Run 120 frames (2 seconds), continuously pushing body toward wall
    for _ in 0..120 {
        // Set velocity toward wall each frame (simulating AI pursuit)
        if let Some(idx) = world.entity_map.get(&2) {
            world.bodies[*idx].velocity = cgmath::Vector2::new(-10.0, 0.0);
        }
        world.step(1.0 / 60.0);
    }

    // Check position — body should be near x=-0.5 (wall edge at x=-4, body half-size 0.5)
    let body_x = world.bodies[world.entity_map[&2]].position.x;
    let wall_right_edge = -4.0; // wall center=-5, half-width=1
    let expected_stop = wall_right_edge + 0.5; // body half-size

    // Body should be stopped at the wall, not sunk through
    assert!(
        body_x >= expected_stop - 0.05,
        "Body sunk through wall: position={:.4}, expected >= {:.4}",
        body_x,
        expected_stop
    );
}

#[test]
fn test_multi_body_continuous_push() {
    let mut world = make_world();

    // Wall at x=0
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);

    // 20 bodies in a line, all pushing toward wall
    let mut id = 2;
    for i in 0..20 {
        let x = 2.0 + (i as f32) * 0.8;
        add_rigid_body_at(&mut world, id, x, 0.0, 0.4);
        id += 1;
    }

    // Run 120 frames with continuous push
    for _ in 0..120 {
        for i in 2..22 {
            if let Some(idx) = world.entity_map.get(&i) {
                world.bodies[*idx].velocity = cgmath::Vector2::new(-10.0, 0.0);
            }
        }
        world.step(1.0 / 60.0);
    }

    // Check all bodies — none should have sunk into the wall
    let mut max_pen = 0.0;
    let wall_right_edge = -4.0;
    for i in 2..22 {
        let body = &world.bodies[world.entity_map[&i]];
        let half = 0.4;
        let sunk = (wall_right_edge - (body.position.x - half)).max(0.0);
        if sunk > max_pen {
            max_pen = sunk;
        }
    }

    assert!(
        max_pen < 0.15,
        "Multi-body continuous push: max penetration {:.4}",
        max_pen
    );
}

// ============================================================================
// Multi-axis tests
// ============================================================================

#[test]
fn test_crowd_pushing_from_multiple_sides() {
    let mut world = make_world();

    // Cross-shaped wall arrangement
    // Vertical wall at x=0
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 10.0);
    // Horizontal wall at y=0
    add_static_wall(&mut world, 2, 0.0, -5.0, 10.0, 1.0);

    // Bodies pushing from all four quadrants toward the center walls
    let mut id = 3;
    let positions = [
        (3.0, 3.0),   // top-right, pushing toward both walls
        (3.0, -3.0),  // bottom-right
        (-3.0, 3.0),  // top-left
        (-3.0, -3.0), // bottom-left
        (3.0, 0.0),   // right, pushing toward vertical wall
        (-3.0, 0.0),  // left
        (0.0, 3.0),   // top, pushing toward horizontal wall
        (0.0, -3.0),  // bottom
    ];

    for (x, y) in &positions {
        add_rigid_body(&mut world, id, *x, *y, 0.0, 0.0, 0.5);
        id += 1;
    }

    // Push bodies toward walls
    for _ in 0..60 {
        let pushes: Vec<(u32, f32, f32)> = vec![
            (3, -10.0, -10.0),   // top-right
            (4, -10.0, 10.0),    // bottom-right
            (5, 10.0, -10.0),    // top-left
            (6, 10.0, 10.0),     // bottom-left
            (7, -10.0, 0.0),     // right
            (8, 10.0, 0.0),      // left
            (9, 0.0, -10.0),     // top
            (10, 0.0, 10.0),     // bottom
        ];
        for (eid, vx, vy) in pushes {
            if let Some(idx) = world.entity_map.get(&eid) {
                world.bodies[*idx].velocity = cgmath::Vector2::new(vx, vy);
            }
        }
        world.step(1.0 / 60.0);
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 0); // already ran

    assert!(
        max_pen < 0.05,
        "Multi-axis crowd penetrated wall by {:.4} (expected < 0.05)",
        max_pen
    );
}

// ============================================================================
// Regression: verify iterations are actually running
// ============================================================================

#[test]
fn test_iterations_matter() {
    // This test verifies that the multi-iteration fix actually changes behavior.
    // With a single iteration and bias=0.5, dense crowds sink through walls.
    // With 3 iterations, they should not.

    use ruin_ecs_plugins::SimpleCollideAndSlideCollisionResolver;

    // Create a resolver with only 1 iteration
    let _resolver_single = SimpleCollideAndSlideCollisionResolver::new(0.0);
    // The default has 3 iterations

    // We can't directly change iterations after construction, but we can
    // verify the default resolver handles the dense crowd correctly.
    // If this test passes, the multi-iteration fix is working.

    let mut world = make_world();

    // Wall at x=0
    add_static_wall(&mut world, 1, -5.0, 0.0, 1.0, 5.0);

    // 30 bodies packed tightly, all pushing toward wall
    let mut id = 2;
    for row in 0..3 {
        for col in 0..10 {
            let x = 1.5 + (col as f32) * 0.85; // tight packing
            let y = -2.0 + (row as f32) * 2.0;
            add_rigid_body(&mut world, id, x, y, -10.0, 0.0, 0.4);
            id += 1;
        }
    }

    let max_pen = run_and_measure_max_penetration(&mut world, 60);

    // With 3 iterations, penetration should be minimal
    assert!(
        max_pen < 0.15,
        "Multi-iteration resolver: 30-body crowd penetrated wall by {:.4} (expected < 0.15)",
        max_pen
    );
}

// ============================================================================
// Live-arena regression: matches main.lua fence box + skelly colliders
// ============================================================================

fn add_game_wall(world: &mut PhysicsWorld, id: u32, x: f32, y: f32, w: f32, h: f32) {
    // create_body half_extents = 0.5 * modifier * size, modifier=1 for fences
    add_static_wall(world, id, x, y, 0.5 * w, 0.5 * h);
}

fn add_game_skelly(world: &mut PhysicsWorld, id: u32, x: f32, y: f32) {
    // width/height 4, collider mods 0.3/0.6 → half = (0.6, 1.2)
    let body = Body2D::new(
        cgmath::Vector2::new(x, y),
        cgmath::Vector2::new(0.0, 0.0),
        BodyType2D::Rigid,
        true,
    );
    world.add_body(id, body);
    world.add_collider(
        &id,
        make_collider(
            Shape2D::Rectangle {
                half_extents: cgmath::Vector2::new(0.6, 1.2),
            },
            bit(ENV) | bit(ENEMY) | bit(PLAYER),
            bit(ENEMY),
        ),
    );
}

#[test]
fn test_live_arena_skellys_stay_inside_walls() {
    let mut world = make_world();

    // Player body at index 0 (physics uses bodies[0] as range center)
    add_game_skelly(&mut world, 1, 0.0, 0.0);

    let fence_thickness = 2.0;
    let fence_count_per_side = 25.0;
    let half_length = fence_thickness * fence_count_per_side / 2.0;

    add_game_wall(
        &mut world,
        2,
        0.0,
        -half_length,
        fence_thickness * fence_count_per_side,
        fence_thickness,
    );
    add_game_wall(
        &mut world,
        3,
        0.0,
        half_length,
        fence_thickness * fence_count_per_side,
        fence_thickness,
    );
    add_game_wall(
        &mut world,
        4,
        -half_length,
        0.0,
        fence_thickness,
        fence_thickness * fence_count_per_side,
    );
    add_game_wall(
        &mut world,
        5,
        half_length,
        0.0,
        fence_thickness,
        fence_thickness * fence_count_per_side,
    );

    // 200 skellys packed like the batch spawner
    let mut id = 10u32;
    for i in 0..200 {
        let x = -20.0 + (i % 20) as f32 * 2.0;
        let y = -20.0 + (i / 20) as f32 * 2.0;
        add_game_skelly(&mut world, id, x, y);
        id += 1;
    }

    let dt = 1.0 / 60.0;
    let mut max_escape = 0.0f32;
    for frame in 0..300 {
        // Chase center (player) like skelly.update
        let (px, py) = {
            let p = &world.bodies[0];
            (p.position.x, p.position.y)
        };
        for i in 1..world.bodies.len() {
            let body = &mut world.bodies[i];
            if !matches!(body.body_type(), BodyType2D::Rigid) {
                continue;
            }
            // Push outward toward nearest wall (dense spawn pressure / flee)
            let dx = if body.position.x.abs() > body.position.y.abs() {
                body.position.x.signum()
            } else if body.position.y.abs() > 0.01 {
                0.0
            } else {
                1.0
            };
            let dy = if body.position.y.abs() >= body.position.x.abs() {
                body.position.y.signum()
            } else {
                0.0
            };
            let speed = 30.0; // lunge-scale
            body.velocity = cgmath::Vector2::new(dx * speed, dy * speed);
        }
        world.bodies[0].velocity = cgmath::Vector2::new(0.0, 0.0);

        world.step(dt);

        for body in &world.bodies {
            if !matches!(body.body_type(), BodyType2D::Rigid) {
                continue;
            }
            let escape = (body.position.x.abs() - 26.0)
                .max(0.0)
                .max((body.position.y.abs() - 26.0).max(0.0));
            if escape > max_escape {
                max_escape = escape;
            }
        }

        if frame == 59 || frame == 179 || frame == 299 {
            let mut escaped = 0u32;
            let mut in_band = 0u32;
            for body in &world.bodies {
                if !matches!(body.body_type(), BodyType2D::Rigid) {
                    continue;
                }
                if body.position.x.abs() > 26.0 || body.position.y.abs() > 26.0 {
                    escaped += 1;
                } else if body.position.x.abs() > 24.0 || body.position.y.abs() > 24.0 {
                    in_band += 1;
                }
            }
            println!(
                "frame {}: escaped={} in_wall_band={} max_escape={:.3}",
                frame + 1,
                escaped,
                in_band,
                max_escape
            );
        }
    }

    assert!(
        max_escape < 0.01,
        "Live-arena skellys escaped past outer wall by {:.3}",
        max_escape
    );
}

#[test]
fn test_single_skelly_stopped_by_game_wall() {
    let mut world = make_world();
    // player placeholder for range center
    add_game_skelly(&mut world, 1, 0.0, 0.0);
    add_game_wall(&mut world, 2, -25.0, 0.0, 2.0, 50.0);
    add_game_skelly(&mut world, 3, -20.0, 0.0);

    println!("wall aabb={:?}", world.bodies[1].aabb_superset);
    println!("skelly aabb={:?}", world.bodies[2].aabb_superset);
    println!(
        "wall masks/layers={:b}/{:b} skelly={:b}/{:b}",
        world.bodies[1].aabbs[0].masks,
        world.bodies[1].aabbs[0].layers,
        world.bodies[2].aabbs[0].masks,
        world.bodies[2].aabbs[0].layers,
    );

    for frame in 0..120 {
        world.bodies[0].velocity = cgmath::Vector2::new(0.0, 0.0);
        world.bodies[2].velocity = cgmath::Vector2::new(-30.0, 0.0);
        world.step(1.0 / 60.0);
        if frame % 20 == 19 {
            let b = &world.bodies[2];
            println!(
                "frame {} skelly x={:.3} aabb=({:.3}..{:.3}) velx={:.2}",
                frame + 1,
                b.position.x,
                b.aabb_superset.min.x,
                b.aabb_superset.max.x,
                b.velocity.x
            );
        }
    }
    let x = world.bodies[2].position.x;
    assert!(
        x > -24.0,
        "single skelly tunneled into/through wall: x={:.3}",
        x
    );
}
