# Review — T-001-02 sky-and-sun

## Summary of Changes

### Files Modified

**src/renderer/sky.rs** — Added bloom bleed feature and unit tests
- Added `draw_bloom_bleed()` public function (25 lines) — renders a 6-row accent-colored glow strip at the bottom of the sky region, alpha-blended over the existing gradient
- Added `blend_alpha()` private helper (10 lines) — standard alpha compositing
- Added `#[cfg(test)] mod tests` (50 lines) — 8 unit tests covering HSL→RGB, lerp_u8, and blend_alpha
- Total: 99 → 185 lines (+86 lines)

**src/renderer/mod.rs** — Wired bloom bleed into render pipeline
- Added one line: `sky::draw_bloom_bleed(...)` call between sun and terrain passes
- Total: 88 → 90 lines (+2 lines)

### Files NOT Modified
- No other files changed. All pre-existing sky functionality (gradient, sun, hue rotation, pulsing) was already correctly implemented.

## Acceptance Criteria Coverage

| Criterion | Status | Evidence |
|---|---|---|
| `draw_sky` fills sky with gradient zenith→horizon | ✅ Pre-existing | sky.rs:8–29 |
| Gradient uses correct HSL→RGB conversion | ✅ Tested | `test_hsl_to_rgb_primaries/secondaries/achromatic` |
| VelocityDemon sky hue rotation at 1°/s | ✅ Pre-existing | sky.rs:100–101 |
| `draw_sun` at (0.72w, horizon_y-40) complement hue | ✅ Pre-existing | sky.rs:38–45 |
| Sun radius pulses ±3px at (cpm*0.4).min(3.0) Hz | ✅ Pre-existing | sky.rs:40–41 |
| 6px bloom bleed of road accent at sky bottom | ✅ **New** | sky.rs:61–86 |
| HSL helpers correct and tested | ✅ **New** | 8 unit tests in sky.rs |

## Test Coverage

**8 unit tests added**, all passing:
- HSL→RGB: 3 tests (primaries, secondaries, achromatic) — validates the core color conversion against known RGB values
- HSL dark horizon: 1 test — validates the specific low-lightness color used at the horizon
- lerp_u8: 2 tests — boundary values and clamping
- blend_alpha: 2 tests — fully transparent and fully opaque edge cases

**Not tested (by design):**
- Rendering output of `draw_sky`, `draw_sun`, `draw_bloom_bleed` — these are visual and verified by running the app
- VelocityDemon hue rotation — would require constructing a full WorldState; the logic is a simple conditional and modular arithmetic

## Open Concerns

1. **HSL→RGB duplication**: The `hsl_to_rgb` function is duplicated across sky.rs, road.rs, effects.rs, and git/seed.rs. All copies are correct and identical. Extracting to a shared utility module would be a worthwhile cleanup but is out of scope for this ticket.

2. **Bloom bleed visual tuning**: The alpha range (0–80) and color parameters (s=0.9, l=0.35) were chosen to produce a subtle glow. These may need visual tuning after seeing the effect in context with different repo hues. The parameters are easy to adjust.

3. **Pre-existing clippy warnings**: 3 warnings exist in font.rs and hud.rs (not related to this ticket). No new warnings introduced.

4. **Pre-existing unused assignment warning**: `proto` in main.rs:72. Not related to this ticket.
