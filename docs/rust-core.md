# Rust Core Reference

## Overview

The Rust core of Ruin is a **2D game engine** built on `winit` (windowing) and `wgpu` (graphics). It implements a custom ECS, a fixed-timestep physics pipeline with spatial-hash collision detection, and a Lua scripting bridge via `mlua`.

## Crate Breakdown

### 1. `ruin_engine` — The Game Engine

**File:** `crates/ruin_engine/src/engine.rs`

The central orchestrator. Implements `winit::ApplicationHandler` and drives the entire game loop.

**State:**
| Field | Purpose |
|-------|---------|
| `world: World` | ECS world — all entities and components |
| `physics: PhysicsWorld` | Physics simulation (bodies, colliders, collision pipeline) |
| `graphics: Graphics2D` | WGPU rendering backend |
| `canvas: Canvas` | UI/canvas scene management |
| `camera2d_config: Camera2DConfig` | Camera follow settings (zoom, look-ahead, smoothing) |
| `lua_context: LuaExtendedExecutor` | Lua runtime bridge |
| `physics_accumulator` | Fixed-timestep accumulator (300 Hz) |
| `fps` | FPS counter for telemetry |

**Key Methods:**
- `setup()` — Exposes ~20 Rust functions to Lua via the `engine` global table
- `update(dt)` — Physics stepping + Lua `ENGINE_update` callback
- `tick_game()` — Frame pacing, physics, camera, render
- `screen_to_world(loc)` — Screen pixel → world coordinate conversion
- `create_body(data)` — Lua-facing entity creation (position, health, animations, colliders)

### 2. `ruin_ecs` — Entity Component System

**Path:** `crates/ruin_ecs/src/`

A flat HashMap-based ECS. Entities are `u32` IDs; components are stored in per-type `HashMap`s.

#### Components (`world.rs`)

| Component | Storage | Purpose |
|-----------|---------|---------|
| `Transform2D` | `HashMap<Entity, Transform2D>` | Position, scale, shape, rotation |
| `AnimationComponent` | `HashMap<Entity, AnimationComponent>` | Sprite frames, current frame index/timer |
| `ActionStateComponent` | `HashMap<Entity, ActionStateComponent>` | Current animation state (idle, run, dash, etc.) |
| `HealthComponent` | `HashMap<Entity, HealthComponent>` | Current/total health |
| `FlipComponent` | `HashMap<Entity, FlipComponent>` | X/Y flip state for mirroring sprites |

#### Physics (`physics_2d/`)

**`body_2d.rs`** — Core physics structures:
- `Body2D` — Position, velocity, colliders, AABBs, body type (Rigid/Static/Kinematic/Trigger)
- `Area2D` — Individual collider with shape, offset, layer/mask bits, active state
- `AABB` — Axis-aligned bounding box with overlap, merge, center operations
- `PhysicsWorld` — Manages bodies, entity→index mapping, and delegates to collision detector/resolver

**Physics pipeline per `step(dt)`:**
```
1. Integrate: velocity → position for all bodies
2. Broad Phase: GridSpaceCollisionDetector finds potential pairs
3. Narrow Phase: AABB overlap test + layer/mask filtering
4. Resolution: SimpleCollideAndSlideCollisionResolver applies MTV
```

**`collision_handler.rs`** — Traits:
- `CollisionDetector` — `broad_phase()` + `narrow_phase()`
- `CollisionResolver` — `resolve(bodies, collisions)`

**`raycast.rs`** — Ray-vs-AABB intersection test (Slab method). Returns hit point, normal, distance, and inside/outside flag.

#### Animation (`animation.rs`)

- `Animation` — Sprite sheet reference, loop flag, frame list
- `SpriteFrame` — UV coordinates, shape, hitboxes, hurtboxes, duration
- `animation_system_update_frames(world, dt)` — Advances frame timer, wraps on loop, updates `current_frame`

#### World (`world.rs`)

- `World::new_entity()` — Returns next sequential ID
- `World::extract_render_queue_2d()` — Collects all visible entities into `RenderQueue2D` (opaque + transparent lists), sorted by `-position.y`
- `World::update_positions(phys_positions)` — Syncs physics positions → transform positions
- Area management: `insert_area_2d`, `toggle_area`, `update_area_masks_and_layers`

### 3. `ruin_graphics` — WGPU Rendering

**Path:** `crates/ruin_graphics/src/lib.rs`

Defines the `Graphics` trait implemented by `Graphics2D`.

**`Graphics` trait methods:**
- `render(world, canvas, physics)` — Main render call
- `resize(width, height)` — Handle window resize
- `load_texture_from_path(id, path)` → `Handle<ImageTexture>`
- `update_camera()` — Apply camera matrix
- `move_camera_for_follow(dt, position, velocity, ...)` — Camera follow with look-ahead
- `get_camera_info()` → `CameraInfo` (zoom, position)
- `process_camera_event(&WindowEvent)` — Independent camera input handling

### 4. `ruin_camera` — 2D Camera

**Path:** `crates/ruin_camera/src/lib.rs`

`Camera2D` with **look-ahead** behavior:
- Smoothly follows a target position with exponential interpolation
- Offsets the camera forward based on velocity (look-ahead) for a "leading" feel
- Handles direction changes (resets look-ahead when direction flips)
- `build_matrix()` → `Matrix4<f32>` orthographic projection

**Configurable:** zoom, initial position, smooth factor, look-ahead distance & speed, screen dimensions.

### 5. `ruin_canvas` — UI Scene Management

**Path:** `crates/ruin_canvas/src/lib.rs`

Manages UI overlay scenes (main menu, HUD, buttons).

- `Canvas` — Holds multiple `CanvasView`s, tracks active views
- `CanvasView` — List of `CanvasNode`s (UI elements with position, scale, animation)
- `parse_canvas_view_from_lua(table)` — Builds canvas from Lua table structure
- `extract_render_queue_2d()` — Adds canvas elements to the render queue (always transparent)

### 6. `ruin_assets` — Asset Management

**Path:** `crates/ruin_assets/src/`

Handle-based asset caching system.

- `Asset` trait — Marker for types managed by the cache
- `AssetCache<T>` — HashMap-based cache with path-based deduplication
- `Handle<T>` — Copy/Clone ID into an asset (index-based, type-safe via PhantomData)
- `ImageTexture` — WGPU texture + sampler + view (loaded from PNG)
- `ImageBindGroup` — WGPU bind group wrapping an image texture

### 7. `ruin_lua_runtime` — Lua Bridge

**Path:** `crates/ruin_lua_runtime/src/lib.rs`

Two executors:
- `LuaScriptor` — Runs individual Lua scripts (e.g., `setup.lua`) and returns a table
- `LuaExtendedExecutor` — Full Lua runtime with package path setup, global `engine` table

**Key utilities:**
- `get_function(name)` — Retrieve Lua function by name
- `table_to_vec_8(table)` — Convert Lua table to `[bool; 8]` for layer/mask bits
- `rust_collisions_to_lua_2d(collisions)` — Convert collision pairs to Lua table
- `create_table()` — Factory for empty Lua tables

### 8. `ruin_ecs_plugins` — Collision Plugins

**Path:** `crates/ruin_ecs_plugins/src/`

#### `GridSpaceCollisionDetector` (active)
- Spatial hash grid with configurable tile size and radius
- Separates static and dynamic bodies into separate maps
- Only queries tiles near the player position
- Layer/mask filtering in broad phase (no separate narrow phase)

#### `SimpleCollideAndSlideCollisionResolver` (active)
- MTV (Minimum Translation Vector) based resolution
- Directional anchoring to prevent sliding on walls
- Two-pass: static collisions first, then dynamic-dynamic
- Velocity clamping along collision normal

#### `BvhCollisionDetector` (unused)
- BVH tree construction from AABBs
- Broad phase only (narrow phase returns empty)
- Currently a placeholder — not wired into the engine

### 9. `ruin_bvh` — Bounding Volume Hierarchy

**Path:** `crates/ruin_bvh/src/lib.rs`

Basic BVH implementation:
- Recursive median-split on longest axis
- Leaf nodes store user data
- Not currently used in the active collision pipeline

### 10. `ruin_bitmaps` — Bit Utilities

**Path:** `crates/ruin_bitmaps/src/lib.rs`

- `vecbool_to_u8([bool; 8])` — Convert bool array to u8 bitmask
- `masks_overlap_layers(a, b)` — Check if two bitmasks intersect

### 11. `ruin_player_controller` — Input Mapping

**Path:** `crates/ruin_player_controller/src/lib.rs`

Maps `winit` key codes and mouse buttons to string identifiers for Lua consumption.

### 12. `ruin_debug` — Debug Utilities

**Path:** `crates/ruin_debug/src/lib.rs`

- `Debug` struct with enabled/disabled flag
- `debug_log!` macro — Conditional println!

## Data Flow: Entity Lifecycle

```
Lua: ruin.load()
  │
  ├─ engine.create_body(entity_data)
  │     │
  │     ├─ World::new_entity() → entity ID
  │     ├─ World::animations.insert(entity, AnimationComponent)
  │     ├─ World::transforms_2d.insert(entity, Transform2D)
  │     ├─ World::health_bars.insert(entity, HealthComponent)
  │     ├─ World::action_states.insert(entity, ActionStateComponent)
  │     ├─ PhysicsWorld::add_body(entity, Body2D)
  │     └─ PhysicsWorld::add_collider(entity, Area2D)
  │
  ▼
Per frame:
  │
  ├─ ENGINE_update(dt) → Lua game logic
  │     ├─ Set velocity via engine.set_velocity_2d()
  │     └─ Change state via engine.set_state()
  │
  ├─ Physics step (300 Hz):
  │     ├─ Integrate velocities
  │     ├─ Collision detection (grid broad + AABB narrow)
  │     └─ Collision resolution (MTV + slide)
  │
  ├─ World::update_positions() — sync physics → transforms
  ├─ animation_system_update_frames() — advance sprite frames
  │
  └─ Graphics::render():
        ├─ World::extract_render_queue_2d() — build render list
        ├─ Canvas::extract_render_queue_2d() — add UI elements
        └─ WGPU render pass with camera matrix
```

## Key Types Summary

| Type | Crate | Purpose |
|------|-------|---------|
| `Engine` | ruin_engine | Main game loop, Lua bridge, state holder |
| `World` | ruin_ecs | ECS world — component storage, render queue |
| `PhysicsWorld` | ruin_ecs | Rigid body simulation, collision pipeline |
| `Body2D` | ruin_ecs | Position, velocity, colliders, AABBs |
| `Area2D` | ruin_ecs | Individual collider (shape + offset + layer/mask) |
| `Transform2D` | ruin_ecs | Position, scale, shape |
| `AnimationComponent` | ruin_ecs | Sprite frame animation state |
| `Canvas` | ruin_canvas | UI overlay scene management |
| `Camera2D` | ruin_camera | Follow camera with look-ahead |
| `Graphics2D` | ruin_graphics | WGPU 2D renderer |
| `AssetCache<T>` | ruin_assets | Handle-based asset caching |
| `Handle<T>` | ruin_assets | Copyable reference to cached asset |
| `LuaExtendedExecutor` | ruin_lua_runtime | Lua runtime + FFI bridge |
