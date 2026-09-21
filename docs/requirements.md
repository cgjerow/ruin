# Performance Requirements: High-Entity-Count 2D Engine

## Implementation Status

### ✅ Completed
- **NFR-3**: `physics_range` added to `EngineConfig` and `PhysicsWorld`. Default: 30.0 world units.
- **CollisionDetector trait**: Updated to accept `(center, range)` in `broad_phase()` and `(position, range)` in `update_player_position()`.
- **GridSpaceCollisionDetector**: Tier filtering implemented — Tier 3 (both out-of-range, entity-entity) pairs are skipped.
- **BvhCollisionDetector**: Matching tier filtering on broad phase.
- **Runtime API**: `engine.set_physics_range(radius)` exposed via Lua for hot adjustment.
- **Helper functions**: `body_in_range()` and `is_terrain()` for tier classification.
- Physics tick rate reduced from 300 Hz to 60 Hz (configurable via `physics_fps`).
- Grid rebuild optimized: only touches bodies within range + 1 tile margin.

### 🔄 In Progress
- Collision pair capping at 1024 (NFR-4).
- Incremental grid rebuild (NFR-4).
- BVH-based broad phase to replace O(n²) brute-force loop.

### 📋 Planned
- Entity pool to avoid allocation churn (FR-4).
- Resolution iteration capping (FR-3).

## Profiling Results (1500 bodies, 60 Hz)

| Phase | Time | % of Physics | Notes |
|-------|------|-------------|-------|
| Integrate | ~0.15 ms | ~1% | Negligible — all bodies |
| Broad phase | ~3–5 ms | ~85% | **Dominant bottleneck** — O(n²) brute force in grid detector |
| Narrow phase | — | (incl.) | Clone only, fast |
| Resolve | ~0.5–0.8 ms | ~10% | MTV + velocity clamping + static re-test |
| **Total** | **~5–6 ms** | | Well within 16.7 ms (60 FPS) budget |

### Broad Phase Breakdown

| Metric | Value | Notes |
|--------|-------|-------|
| Bodies in range | ~1400+ | Most entities spawn within range |
| Bodies filtered | ~100 | Few outside range |
| Grid tiles populated | 400–500 | Tile size = entity size |
| Pair checks (potential) | 50K–80K | Grid cell pair enumeration |
| Visited-skip (dedup) | 15K–25K | Already-seen pairs skipped |
| **Pairs added (actual)** | **3K–5K** | AABB overlap + layer/mask + tier filtering |
| Grid insertion | 0.5–0.8 ms | HashMap ops |
| Pair enumeration | 2.5–4.2 ms | **Main cost** — nested loops |

### Key Insights

1. **Broad phase is ~85% of physics time** — the grid's nested pair enumeration over tiles is the bottleneck.
2. **Range filtering has limited impact** — most entities spawn within the 30-unit range, so almost all bodies get inserted.
3. **Tier 3 filtering saves work** — skipping entity-entity pairs for bodies both out-of-range reduces pair count significantly.
4. **Collision resolution is cheap** — <1 ms for 3K–5K pairs, well within budget.
5. **Integration is trivial** — <0.2 ms for 1500 bodies, not a concern.
6. **FPS stays stable at ~120** — physics uses ~5.5 ms of 8.33 ms budget at 60 Hz (66% utilization). At 120 FPS render, physics uses ~5.5 ms of 16.67 ms (33% utilization).

| Phase | Time | % of Physics | Notes |
|-------|------|-------------|-------|
| Integrate | 0.05 ms | 0.8% | Negligible — all bodies |
| Broad phase | 5.9–6.9 ms | 92% | **Dominant bottleneck** |
| Narrow phase | — | (incl.) | Clone only, fast |
| Resolve | 0.37–0.48 ms | 6% | MTV + velocity clamping |
| **Total** | **6.3–7.4 ms** | | Well within 16.7 ms (60 FPS) budget |

### Broad Phase Breakdown

| Metric | Value | Notes |
|--------|-------|-------|
| Bodies in range | ~503 | Most entities spawn within range |
| Bodies filtered | ~2 | Very few outside range |
| Grid tiles populated | 160–179 | Tile size = entity size |
| Pair checks (potential) | 11,000–14,800 | Grid cell pair enumeration |
| Visited-skip (dedup) | 3,000–4,800 | Already-seen pairs skipped |
| **Pairs added (actual)** | **2,000–2,500** | AABB overlap + layer/mask |
| Grid insertion | 0.29–0.32 ms | Fast — HashMap ops |
| Pair enumeration | 5.6–6.4 ms | **Main cost** — nested loops |

### Key Insights

1. **Broad phase is 92% of physics time** — the grid's nested pair enumeration over tiles is the bottleneck.
2. **Range filtering has limited impact** — all 500 skellies spawn within the 30-unit range, so almost all bodies get inserted.
3. **Collision resolution is cheap** — 0.4 ms for 2000+ pairs, well within budget.
4. **Integration is trivial** — 0.05 ms for 500 bodies, not a concern.
5. **FPS stays stable at ~120** — physics uses ~6.5 ms of 16.7 ms budget (39% utilization).

## Vision

This engine must support **Vampire Survivors-scale** entity counts: hundreds of active entities with full physics simulation and collision handling, maintaining stable frame rates under worst-case conditions.

## Target Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| **Active entities** | 500–1000 | All with physics bodies, colliders, and animations |
| **Collision pairs per frame** | Up to ~50,000 | Worst case: dense cluster of entities overlapping |
| **Physics frequency** | 60 Hz | Matched to frame rate; 300 Hz is unnecessary overhead (was 300 Hz, reduced to 60 Hz) |
| **Target frame time** | ≤ 16.67 ms | 60 FPS with headroom |
| **Physics budget** | ≤ 6 ms | ~35% of frame time |
| **Collision detection budget** | ≤ 2 ms | Broad + narrow phase |
| **Collision resolution budget** | ≤ 2 ms | MTV + velocity clamping |
| **Render budget** | ≤ 8 ms | Sprite batching + WGPU |
| **Lua script budget** | ≤ 2 ms | Game logic callbacks |
| **Memory** | < 500 MB | All assets + entity data |

## Functional Requirements

### FR-1: Physics Simulation

- All active entities must have full rigid body simulation (position, velocity, integration)
- Bodies must support Rigid, Static, Kinematic, and Trigger types
- Fixed timestep integration with accumulator for deterministic physics
- Position sync from physics bodies to ECS transform components each step

### FR-2: Collision Detection

- **Broad phase** must scale sub-quadratically: O(n log n) or O(n) expected
- **Narrow phase** must filter by layer/mask before computing MTV
- Must use spatial partitioning (grid or BVH) — no brute-force O(n²)
- Spatial partition must only iterate entities near the **player camera view**, not the entire world
- Must support configurable tile size and query radius
- Must deduplicate collision pairs (A→B reported only once)

### FR-3: Collision Resolution

- Must resolve static-dynamic collisions before dynamic-dynamic (priority ordering)
- MTV-based resolution with directional anchoring (wall slide prevention)
- Velocity clamping along collision normals
- **Must cap resolution iterations** — if resolution can't settle within N passes, skip remaining pairs to preserve frame budget

### FR-4: Entity Management

- Entity creation/destruction must be O(1) amortized
- Entity pools to avoid allocation churn during spawning bursts
- Bodies must be activatable/deactivatable without reallocation
- World unload must be instant (clear + reset, no per-entity cleanup)

### FR-5: Rendering

- Sprite rendering must use **batched draw calls** — no per-entity draw
- Y-sorted rendering for pseudo-3D depth
- Virtual resolution scaling (e.g., 320×160 → 1280×720)
- Canvas/UI overlay rendered on top, sorted separately

### FR-6: Lua Integration

- All engine APIs called from Lua must be **O(1)** or batched
- Collision callbacks must pass **only relevant data** — no large table allocations per frame
- Lua `update` and `after_physics` callbacks must not block the physics pipeline

## Non-Functional Requirements

### NFR-1: Frame-Time Stability

- **No frame may exceed 33 ms** (30 FPS floor) under target load
- Physics step must never exceed its 4 ms budget — if it would, **skip the step** or **reduce collision pairs**
- Lua `update` must not exceed 2 ms — if it would, **skip non-essential logic**

### NFR-2: Memory Efficiency

- Entity data must use **contiguous arrays** (Structure of Arrays) where possible for cache efficiency
- No per-entity heap allocations after initialization
- Collision pair lists must use pre-allocated buffers with reuse

### NFR-3: Configurable Physics Range

- Physics simulation range must be configurable per-engine, matching the camera's view range pattern
- Range defined as a radius (in world units) around the player/camera center
- **Integration runs on ALL bodies** — off-screen entities still move via velocity → position, so they're where they should be when the camera approaches
- **Collision detection is tiered** — entity-terrain pairs are always resolved; entity-entity pairs are skipped when both bodies are out-of-range (Tier 3)
- Range must be settable via `EngineConfig` and adjustable at runtime (e.g., `engine.set_physics_range(radius)`)
- Default range: 30.0 world units (60×60 unit square)

### NFR-4: Spatial Partition Accuracy

- Grid must only query tiles within the **configured physics range**, not the entire world
- Tile size must be configurable but default to entity size × 2 (minimizes cross-tile entities)
- Grid rebuild must be incremental — only update tiles for changed entities within the range

### NFR-4: Collision Pair Capping

- Maximum collision pairs per step: **1024** (configurable)
- When cap is reached, prioritize pairs involving the **player** and closest entities
- Remaining pairs are deferred to next physics step
- **Note: Not yet implemented.** Current code processes all pairs from broad phase. This is a planned optimization.

## Acceptance Criteria

| # | Test | Pass Condition |
|---|------|---------------|
| AC-1 | Spawn 1000 entities in arena, all overlapping | Stable 60 FPS for 60 seconds |
| AC-2 | Spawn/despawn 100 entities mid-frame | No frame spike > 20 ms |
| AC-3 | Move player through dense entity cluster | No physics step > 4 ms |
| AC-4 | World unload + reload | < 10 ms total |
| AC-5 | Lua spawns 50 entities per frame for 10 seconds | FPS stays above 45 |
| AC-6 | Render 1000 animated sprites at virtual resolution | Render pass < 8 ms |

### NFR-5: Physics Range vs Camera Decoupling

- Physics range and camera view range are **independent** settings
- Camera can zoom in/out without affecting physics simulation
- Use case: tight camera for clarity, large physics range for off-screen entity interactions (e.g., projectiles, AoE)

## Out of Scope (v1)

- 3D rendering
- Networked multiplayer
- Soft-body physics
- Continuous collision detection (CCD)
- GPU-based physics (compute shaders)
