# T-003-01 Research: Sprite Rendering

## Scope

Depth-projected roadside object rendering. Objects in world space `(lane, z_world)` project to screen `(x, y, scale)` and render as colored rects with optional bitmap-font text overlay.

## Existing Implementation

The sprite rasterizer was implemented in commit `273c748`. This research documents the codebase state.

### Key Files

| File | Role |
|------|------|
| `src/renderer/sprites.rs` | `project()`, `draw_sprites()`, `draw_rect()` — the full sprite pipeline |
| `src/renderer/font.rs` | 5×7 bitmap font, `draw_text()`, `draw_char()`, `text_width()` |
| `src/renderer/road.rs` | `road_max_half()`, `horizon_ratio()` — shared constants used by projection |
| `src/renderer/mod.rs` | Calls `sprites::draw_sprites()` at pass 7 in the render pipeline |
| `src/world/objects.rs` | `RoadsideObject` enum (8 variants), `Lane` enum, `ingest_poll_to_queue()` |
| `src/world/mod.rs` | `WorldState` — owns `active_objects: Vec<(Lane, f32, RoadsideObject)>`, spawning logic |
| `src/world/speed.rs` | `VelocityTier` enum — `name()` used by TierGate text |
| `src/git/seed.rs` | `RepoSeed` — `author_colors` map, `accent_hue` |

### Projection Model

`project()` in `sprites.rs:20-56`:
- `z_rel = z_world - camera_z` — relative distance ahead of camera
- Culling: `z_rel <= 0` (behind camera) or `depth_scale < 0.02` (beyond draw distance)
- `depth_scale = (1.0 - z_rel / draw_dist).clamp(0.0, 1.0)` — 0 at horizon, 1 at camera
- `screen_y = lerp(horizon_y, pixel_h, depth_scale)` — linear Y interpolation
- `road_half_here = lerp(ROAD_MIN_HALF, max_half, depth_scale)` — road width at this depth
- `cx = pixel_w/2 + curve_offset * depth_scale²` — quadratic curve shift
- Lane positioning: Left at `cx - road_half * 1.15`, Right at `cx + road_half * 1.15`, Center at `cx`

### Sprite Sizing

All sprites use quadratic scaling: `base * scale * scale`. This matches the spec's OutRun feel.

| Object | Base W | Base H | Notes |
|--------|--------|--------|-------|
| CommitBillboard | 80 | 40 | author_color rect + white text |
| AdditionTower | 12 | sqrt(lines)*4 | author_color, height proportional |
| DeletionShard | 8 | sqrt(lines)*3 | Crimson Rgba(180,30,30) |
| TierGate | 240 | 60 | Neon magenta arch (3 rects) + tier text |
| VelocitySign | 20 | 20 | Yellow diamond + black c/min text |
| Others (fallback) | 16 | 16 | Gray rect |

### Text Rendering

- Text suppressed when `depth_scale < 0.35` — object renders as colored rect only
- `depth_scale >= 0.35`: text at 1× glyph scale (5×7 px per char)
- `depth_scale >= 0.65`: text at 2× glyph scale (10×14 px per char)
- Font: builtin 5×7 bitmap, ASCII 32-126, rendered via `font::draw_text()`

### Sort Order

`draw_sprites()` sorts `active_objects` by `z_world` descending (far first), so near objects overdraw far ones correctly.

### World-Side Object Lifecycle

- `WorldState::ingest_poll()` converts `PollResult` into `RoadsideObject` queue entries
- `WorldState::update()` drains `pending_objects`, assigns lane (alternating L/R, TierGate always Center), places at `camera_z + SPAWN_DISTANCE + spacing*i`
- Despawn: `active_objects.retain(|(_, z, _)| *z > camera_z - DESPAWN_BEHIND)`
- TierGate on tier change: spawned immediately at `camera_z + NEAR_SPAWN`

### Draw Distance

- Normal: 200.0 world units
- VelocityDemon: 240.0 (20% increase)

### Dependencies (T-001-01, T-001-03)

- T-001-01: FrameRenderer / framebuffer — sprites write directly into `ImageBuffer<Rgba<u8>>`
- T-001-03: Road rasterizer — sprites use `road::road_max_half()` and `road::horizon_ratio()` for consistent projection

### Object Variants Not Yet Rendered

`FilePosts`, `IdleAuthorTree`, `SectorPylon` fall through to the generic gray rect fallback (`_ =>` match arm). These are defined in the enum but have no specialized rendering. Not required by acceptance criteria.

### Public API Surface

- `sprites::draw_sprites()` — sole public function, called from `FrameRenderer::render()`
- `sprites::ROAD_MIN_HALF` — re-exported constant (duplicated from `road.rs`)
- `sprites::SpriteScreenPos` — public struct but only used internally

### Test Coverage

No tests exist in `sprites.rs`. The `road.rs` (14 tests) and `font.rs` (5 tests) modules have comprehensive tests that serve as templates. Sprite projection and rendering are entirely untested.

### Constraints

- No heap allocation in render hot path (per performance targets)
- `draw_rect()` uses bounds-checked `put_pixel` per pixel — safe but slower than raw buffer writes
- `active_objects` is a `Vec` that gets sorted every frame — O(n log n) per frame

### Duplicated Constant

`ROAD_MIN_HALF = 8.0` defined in both `road.rs:8` and `sprites.rs:12`. Sprite module uses its own copy rather than importing from road.
