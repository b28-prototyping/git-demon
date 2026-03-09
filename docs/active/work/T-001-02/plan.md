# Plan — T-001-02 sky-and-sun

## Implementation Steps

### Step 1: Add bloom bleed helpers to sky.rs

Add two private helper functions:
- `blend_alpha(bg, fg) -> Rgba<u8>` — alpha composite fg over bg
- `bloom_accent_color(hue, world, alpha) -> Rgba<u8>` — returns the neon accent color at given alpha, with VelocityDemon hue rotation

**Verify:** `cargo build` succeeds (unused function warnings OK at this stage).

### Step 2: Add `draw_bloom_bleed` public function to sky.rs

Implement the 6-row accent glow at the bottom of the sky region:
- Iterate rows `(horizon_y - 6)..horizon_y`, clamped to valid pixel range
- For each row, compute alpha as `((row - start) as f32 / 6.0 * 80.0) as u8`
  - Bottom row (closest to horizon) gets alpha ~80
  - Top row (farthest) gets alpha ~0
- Compute accent color via `bloom_accent_color`
- Alpha-blend over existing sky gradient pixels

**Verify:** `cargo build` succeeds.

### Step 3: Wire bloom bleed into render pipeline (mod.rs)

Add `sky::draw_bloom_bleed(&mut self.fb, w, horizon_y, seed, world);` after the `draw_sun` call and before `draw_terrain`.

**Verify:** `cargo build` succeeds, `cargo run -- --repo .` shows visible glow at horizon.

### Step 4: Add unit tests for HSL helpers

Add `#[cfg(test)] mod tests` at the bottom of sky.rs:

1. `test_hsl_to_rgb_primaries` — assert red=hsl(0,1,0.5), green=hsl(120,1,0.5), blue=hsl(240,1,0.5)
2. `test_hsl_to_rgb_secondaries` — yellow=hsl(60,1,0.5), cyan=hsl(180,1,0.5), magenta=hsl(300,1,0.5)
3. `test_hsl_to_rgb_achromatic` — black=hsl(0,0,0), white=hsl(0,0,1), gray=hsl(0,0,0.5)
4. `test_hsl_to_rgb_dark_horizon` — verify hsl(0, 0.4, 0.08) returns expected dark red
5. `test_lerp_u8_basic` — t=0 returns a, t=1 returns b, t=0.5 returns midpoint
6. `test_lerp_u8_clamp` — extreme values don't overflow

**Verify:** `cargo test` — all tests pass.

### Step 5: Run full quality checks

- `cargo clippy` — no warnings
- `cargo test` — all tests pass
- `cargo build` — clean build

## Testing Strategy

| What | Type | Criteria |
|---|---|---|
| HSL→RGB primaries | Unit | Exact RGB match for 0°,120°,240° at s=1 l=0.5 |
| HSL→RGB secondaries | Unit | Exact RGB match for 60°,180°,300° at s=1 l=0.5 |
| HSL→RGB achromatic | Unit | Black, white, gray at s=0 |
| HSL dark horizon | Unit | Expected dark color for s=0.4 l=0.08 |
| lerp_u8 | Unit | Boundary values and midpoint |
| Bloom bleed rendering | Visual | Visible glow at horizon in running app |
| VelocityDemon rotation | Visual | Hue shifts when tier ≥ 4 |

No integration tests needed — the rendering is visual and verified by running the app.

## Commit Plan

- **Commit 1:** Steps 1–3 (bloom bleed feature)
- **Commit 2:** Step 4 (unit tests)

Or single commit if preferred — changes are small and cohesive.
