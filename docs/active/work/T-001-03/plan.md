# T-001-03 Plan: road-rasterizer-with-curvature

## Step 1: Fix cx wrapping bug

**Change:** In `draw_road`, keep `cx` as `f32` and compute `road_l` using `f32::max(0.0)` instead of casting to `u32` first.

**Lines affected:** 57-61 of `src/renderer/road.rs`

**Before:**
```rust
let cx = (cx_base + curve_shift) as u32;
let road_half = lerp(ROAD_MIN_HALF, max_half, depth);
let road_l = cx.saturating_sub(road_half as u32);
let road_r = (cx + road_half as u32).min(w - 1);
```

**After:**
```rust
let cx = cx_base + curve_shift;
let road_half = lerp(ROAD_MIN_HALF, max_half, depth);
let road_l = (cx - road_half).max(0.0) as u32;
let road_r = ((cx + road_half) as u32).min(w - 1);
```

**Verify:** `cargo build && cargo clippy`

## Step 2: Add test helpers

Add `#[cfg(test)] mod tests` with:
- `test_seed()` — constructs `RepoSeed` with `accent_hue: 180.0`, all fields populated
- `test_world(tier)` — constructs `WorldState` directly with given tier, sets reasonable defaults for z_offset, curve_offset, etc.

**Verify:** `cargo test` — no new tests yet, just helpers compile

## Step 3: Add pure function unit tests

| Test | What it verifies |
|------|-----------------|
| `test_lerp_at_zero` | `lerp(a, b, 0.0) == a` |
| `test_lerp_at_one` | `lerp(a, b, 1.0) == b` |
| `test_lerp_midpoint` | `lerp(0.0, 10.0, 0.5) == 5.0` |
| `test_horizon_ratio_normal` | Returns 0.35 for non-VelocityDemon tiers |
| `test_horizon_ratio_velocity_demon` | Returns 0.37 for VelocityDemon |
| `test_road_max_half_normal` | Returns 480.0 for non-VelocityDemon |
| `test_road_max_half_velocity_demon` | Returns 504.0 for VelocityDemon |
| `test_blend_alpha_opaque` | fg with alpha=255 fully replaces bg |
| `test_blend_alpha_transparent` | fg with alpha=0 preserves bg |
| `test_blend_alpha_half` | Midpoint blend |
| `test_hsl_to_rgb_red` | h=0, s=1, l=0.5 → (255, 0, 0) |
| `test_hsl_to_rgb_green` | h=120, s=1, l=0.5 → (0, 255, 0) |
| `test_hue_to_neon_returns_opaque` | Alpha channel is always 255 |

**Verify:** `cargo test -- road::tests`

## Step 4: Add rendering invariant tests

| Test | What it verifies |
|------|-----------------|
| `test_draw_road_perspective_width` | Road is narrower near horizon than at bottom. Sample road width at y near horizon vs y near bottom. |
| `test_draw_road_stripe_colors_present` | Both `STRIPE_LIGHT` and `STRIPE_DARK` appear in road region |
| `test_draw_road_rumble_colors_present` | Both `RUMBLE_WHITE` and `RUMBLE_RED` appear in framebuffer |
| `test_draw_road_verge_colors_present` | Both `VERGE_A` and `VERGE_B` appear in framebuffer |
| `test_draw_road_curve_shifts_center` | With positive curve_offset, road center at bottom row is shifted right vs zero offset |
| `test_draw_road_velocity_demon_wider` | Road at bottom row is wider with VelocityDemon tier |
| `test_draw_road_no_panic_extreme_curve` | `curve_offset = -80.0` on 100×100 buffer doesn't panic |
| `test_draw_grid_accent_pixels` | After draw_grid, some pixels on road have accent hue color |
| `test_draw_grid_alpha_blended` | Grid pixels are not raw accent color — they're blended with road background |

**Verify:** `cargo test -- road::tests` — all pass

## Step 5: Final verification

- `cargo test` — all 14 existing + new tests pass
- `cargo clippy` — no warnings
- `cargo build` — clean

## Testing Strategy

- All tests are deterministic (no randomness in road rendering)
- Test buffers are small (200×200 or 100×100) for speed
- Rendering tests check invariants (pixel presence, relative widths) not exact positions
- Pure function tests use exact equality where possible, epsilon comparison for floats
