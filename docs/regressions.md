# Regressions

Recorded bugs that were fixed and must not come back. Prefer a failing regression test over folklore when the failure is reproducible in CI.

---

## REG-001: Dense-crowd terrain tunneling

**Status:** Fixed  
**Symptoms:** In densely populated areas (and under strong outward / lunge-scale push), rigid entities were shoved through static arena walls and then free to leave the map. A single body against a wall stopped correctly; crowds did not.  
**Seen in:** Live `cargo run` arena (`main.lua` fence box + skelly batch spawn); reproduced in `crates/ruin_ecs_plugins/tests/collision_tests.rs` (`test_live_arena_skellys_stay_inside_walls`).

### Root causes

Several issues stacked:

1. **Stale AABBs during resolve**  
   `SimpleCollideAndSlideCollisionResolver` moved `Body2D::position` but only `integrate` refreshed world-space AABBs. Multi-pass resolution kept applying the same post-integrate overlaps, so iterations did not behave like Gauss–Seidel.

2. **Inverted wall velocity clamp**  
   After separating a rigid from a static, velocity was clamped with the MTV direction instead of the separation direction. Bodies kept full into-wall speed (position was corrected each frame for a lone body, but stacks kept driving penetration).

3. **Precomputed static pairs only**  
   Broadphase runs once per step. Entity–entity resolution could translate a body that was *not* overlapping terrain at broadphase time all the way through a wall in one solve. By the time a static pass ran, the AABBs no longer overlapped, so no MTV existed and the body stayed outside forever.

4. **Incomplete MTV when wall-anchored**  
   When one rigid was anchored against a wall, zeroing its half of the MTV without giving the full separation to the free body left residual stack pressure that the next shove reused.

### Fix

In `simple_collide_and_slide_collision_resolver.rs` and `Body2D`:

- Call `Body2D::sync_aabbs()` after any resolve position change.
- Seed anchors with a static pass, then each iteration: resolve dynamics; after a body moves, **re-test that body against every static** (`separate_from_all_statics`), not only pairs from the broadphase list; finish with a full rigid-vs-static sweep.
- Clamp rigid velocity along the **separation** direction (out of the wall).
- When an axis is blocked by an anchor, transfer the full MTV onto the free body (or discard if both blocked).
- Bidirectional mask checks: `masks(a)∩layers(b)` or `masks(b)∩layers(a)`.

Supporting broadphase behavior (tiered collision): entity–entity pairs are limited by physics range; **entity–terrain pairs remain global** so off-range bodies still hit walls.

### Regression tests

Run:

```bash
cargo test -p ruin_ecs_plugins --test collision_tests
```

Critical cases:

| Test | What it guards |
|------|----------------|
| `test_single_skelly_stopped_by_game_wall` | Game-scale skelly collider + fence wall; lunge-speed push must stop at inner face |
| `test_live_arena_skellys_stay_inside_walls` | ~200 skellys, real fence box (±25, thickness 2), outward speed 30 — **must not** pass outer face ±26 |
| Dense crowd / continuous-push tests | Stacked rigid-vs-static under lateral pressure |

### Do not regress by

- Moving bodies in the resolver without syncing AABBs.
- Resolving all dynamic pairs for a full iteration before any static correction for *moved* bodies.
- Relying only on the broadphase static-pair list for terrain response mid-solve.
- “Fixing” density by lowering iterations without the per-shove static re-test (lone bodies look fine; crowds tunnel).
