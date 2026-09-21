# Architecture Overview

## System Topology

```
┌─────────────────────────────────────────────────────────────┐
│                         winit EventLoop                      │
│                    (Window + Input Management)               │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                        Engine (Rust)                         │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌───────────┐  │
│  │  Camera  │  │  Graphics │  │   World  │  │  Physics  │  │
│  │  (2D)    │  │   (WGPU)  │  │   (ECS)  │  │  World    │  │
│  └──────────┘  └───────────┘  └──────────┘  └───────────┘  │
│  ┌──────────┐  ┌───────────┐                                │
│  │  Canvas  │  │   Debug   │                                │
│  │  (UI)    │  │           │                                │
│  └──────────┘  └───────────┘                                │
└──────────────────────┬──────────────────────────────────────┘
                       │ mlua (FFI bridge)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                      Lua Runtime                             │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌───────────┐  │
│  │  main.lua│  │  systems  │  │ characters│ │  canvas   │  │
│  │  (game   │  │(physics/  │  │ (NPCs/    │ │  (UI      │  │
│  │   logic) │  │  collisions│ │  player)  │  │  scenes)  │  │
│  └──────────┘  └───────────┘  └──────────┘  └───────────┘  │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌───────────┐  │
│  │  globals │  │  utils    │  │  asset    │  │  math     │  │
│  │          │  │  (json,   │  │  builders │  │           │  │
│  │          │  │  pretty)  │  │           │  │           │  │
│  └──────────┘  └───────────┘  └──────────┘  └───────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## The Rust-Lua Relationship

Ruin uses a **Rust core + Lua game logic** architecture, where:

### Rust owns:
- **The game loop** — frame timing, physics stepping, rendering
- **The ECS (Entity Component System)** — entities, components, world state
- **The physics engine** — rigid body simulation, collision detection & resolution
- **The renderer** — WGPU-based 2D sprite rendering with camera
- **Asset management** — texture loading, handle-based references, caching
- **Input abstraction** — keyboard/mouse events mapped to Lua callbacks

### Lua owns:
- **Game logic** — player movement, enemy AI, dash mechanics, cooldowns
- **Entity creation** — spawning players, enemies, walls, tiles
- **Scene management** — loading/unloading game worlds, UI canvases
- **Collision callbacks** — reacting to collisions with game-specific behavior
- **UI / Canvas rendering** — main menu, buttons, HUD elements

### Communication Pattern

```
Rust Engine ───calls───▶ Lua: ENGINE_update(dt)
Rust Engine ───calls───▶ Lua: ENGINE_after_physics(dt)
Rust Engine ───calls───▶ Lua: ENGINE_on_collision(collisions)
Rust Engine ───calls───▶ Lua: ENGINE_input_event(key, pressed, mouse_pos)
Rust Engine ───calls───▶ Lua: ENGINE_load()

Lua ───calls───▶ Rust: engine.create_body(...)
Lua ───calls───▶ Rust: engine.set_velocity_2d(id, x, y)
Lua ───calls───▶ Rust: engine.set_state(id, state)
Lua ───calls───▶ Rust: engine.flip(id, x, y)
Lua ───calls───▶ Rust: engine.damage(id, amount)
Lua ───calls───▶ Rust: engine.create_canvas_view(...)
Lua ───calls───▶ Rust: engine.get_position_2d(id)
Lua ───calls───▶ Rust: engine.get_health_table(id)
```

The bridge is implemented via `mlua` (Lua 5.4 bound to Rust). Rust exposes functions through a global `engine` table accessible in Lua. Lua callbacks are invoked by Rust at specific game loop points.

## Game Loop

```
winit::ApplicationHandler::about_to_wait()
    │
    ▼
tick_game(dt)
    │
    ├── FPS measurement & frame pacing
    │
    ├── update(dt)                    ← Lua: ENGINE_update(dt)
    │     ├── Physics stepping (60 Hz fixed tick)
    │     │   ├── Integrate (velocity → position)
    │     │   ├── Collision broad phase (tiered spatial grid)
    │     │   ├── Collision narrow phase (AABB overlap + layer/mask)
    │     │   └── Collision resolution (MTV + velocity clamp + static re-test)
    │     ├── Lua collision callbacks
    │     └── Animation frame advancement
    │
    ├── Camera update (follow player with look-ahead)
    │
    └── render(world, canvas, physics)
          ├── Render queue extraction (opaque + transparent)
          ├── WGPU render pass
          └── Swap chain present
```

## Key Design Decisions

1. **Fixed timestep physics** — Physics runs at a fixed 60 Hz rate (16.67ms per step), independent of frame rate. The accumulator ensures deterministic physics regardless of rendering speed. (Previously 300 Hz; reduced to 60 Hz to match frame rate and cut physics work by 80%.)

2. **Tiered spatial hash grid for collision** — `GridSpaceCollisionDetector` uses a tile-based spatial hash with configurable tile size and physics range. Entities are classified into tiers: both-in-range (full collision), one-in-range (full collision), both-out-of-range (entity-entity skipped, entity-terrain still resolved). This avoids O(n²) pairwise checks for off-screen entities while keeping terrain solid everywhere.

3. **Component-based world** — The `World` struct holds flat `HashMap<Entity, Component>` mappings. This is a sparse ECS without query systems; components are accessed directly by entity ID.

4. **Y-sorted rendering** — 2D sprites are sorted by `-position.y` (lower on screen = drawn later = on top), giving a pseudo-3D depth effect.

5. **Virtual resolution** — The game renders to a 320×160 virtual resolution, then scales up to the window size via WGPU, giving a pixel-art aesthetic.

6. **Handle-based asset management** — Textures are loaded once into an `AssetCache` and referenced via `Handle<T>`, avoiding duplicate loads and providing stable IDs.

## Module Dependencies

```
ruin_engine (central)
├── ruin_ecs              ← Entities, components, physics_2d
│   ├── ruin_assets       ← Image textures, asset cache
│   └── ruin_bitmaps      ← Mask/layer bit utilities
├── ruin_graphics         ← WGPU rendering (Graphics2D)
├── ruin_camera           ← Camera2D with look-ahead
├── ruin_canvas           ← UI scene management
├── ruin_lua_runtime      ← Lua bridge (mlua)
├── ruin_player_controller← Input key/button mapping
├── ruin_ecs_plugins      ← Collision detectors & resolver
│   ├── ruin_bvh          ← BVH spatial structure (placeholder, not yet implemented)
│   └── ruin_bitmaps
└── ruin_debug            ← Debug logging macro
```
