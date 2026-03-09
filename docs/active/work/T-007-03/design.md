# Design: T-007-03 parallax-depth-layers

## Approach Options

### Option A: Parallax via camera_z scaling in terrain.rs

Multiply `camera_z` by a parallax factor when computing `z_rel` in each terrain sub-layer. This is a one-line change per layer.

**Islands**: `z_rel = ((base_z - (world.camera_z * 0.4 % z_period)) + z_period) % z_period`
**Clouds**: `z_rel = ((base_z - (world.camera_z * 0.15 % z_period)) + z_period) % z_period`

Pros: Minimal change, obvious semantics, no new abstractions.
Cons: Parallax factor is hardcoded per call site.

### Option B: Parallax as a Camera method

Add `Camera::project_parallax(z_world, parallax_factor, ...)` that internally adjusts `z_rel`.

Pros: Centralizes parallax logic.
Cons: Over-engineered for 2 call sites. The projection itself doesn't change — only the input Z changes. Wrapping this in Camera conflates two concerns.

### Option C: Layer-specific scroll offset in WorldState

Track `island_scroll_z` and `cloud_scroll_z` as separate fields, updated in `WorldState::update()` with `speed * dt * parallax_factor`.

Pros: Decouples layers from camera_z entirely.
Cons: Adds state fields for something that's a pure function of camera_z.

## Decision: Option A

Option A is the right choice because:
1. The change is a pure function of `camera_z` — no state needed
2. Only 2 call sites need modification (islands, clouds)
3. The parallax factor is a property of the content, not the camera
4. It's trivially testable: render at two different camera_z values and verify scroll difference

## Pitch-Based Vertical Shift

For camera pitch integration, each layer's screen_y gets an additive offset:
```
pitch_offset = camera.pitch * (1.0 - parallax_factor) * screen_h * PITCH_SENSITIVITY
```

Where `PITCH_SENSITIVITY` is a tuning constant (~0.5) controlling how much pitch affects the view.

This applies to:
- Stars: pitch_offset with parallax 0.0 → full shift (`pitch * screen_h * 0.5`)
- Sun: pitch_offset with parallax 0.05 → near-full shift
- Clouds: pitch_offset with parallax 0.15
- Islands: pitch_offset with parallax 0.4
- Road/sprites: pitch_offset with parallax 1.0 → zero shift

Since pitch is currently always 0.0, these offsets are zero. When T-007-02 sets pitch, parallax shifts activate automatically.

## Sun Parallax

The sun is currently at fixed screen position `(w * 0.72, horizon_y - 40)`. To give it parallax 0.05, add a tiny camera_z-based horizontal drift:
```
let sun_drift = (camera_z * 0.05) % (w as f32 * 2.0) - w as f32;
let cx = (w as f32 * 0.72 + sun_drift * 0.01) as i32;
```

This makes the sun drift very slowly relative to road motion. However, the ticket says "near-stationary" and the sun already doesn't scroll. The pitch shift alone is sufficient — skip horizontal sun drift to avoid visual disruption.

## Render Order

No changes needed. Current order already has correct layering:
1. Sky (farthest) — parallax 0.0
2. Ocean surface — parallax 1.0 (ground plane)
3. Islands (near-bg, 0.4) + Clouds (mid-bg, 0.15) — drawn on top of ocean
4. Grid — parallax 1.0
5. Sprites — parallax 1.0
6. Effects — screen-space

The clouds/islands draw order within `draw_terrain` doesn't need to change: islands are drawn first (at/below horizon), clouds overlay them (semi-transparent). Clouds having a slower parallax than islands is correct — they're further away.

## Testing Strategy

1. **Parallax rate test**: Render islands at camera_z=0 and camera_z=1000. Then at camera_z=0 and camera_z=2500 (which is 1000/0.4). The second pair should produce the same pixel shift as the first pair. This validates the 0.4 factor.

2. **Cloud vs island differential**: Render at camera_z=0 and camera_z=5000. Measure pixel displacement of clouds vs islands. Clouds should displace less (0.15 vs 0.4 factor).

3. **Existing test preservation**: `test_islands_scroll_with_camera` passes since islands still scroll (just at 0.4x rate). `test_draw_terrain_stays_below_horizon` passes since parallax doesn't change horizon boundary.

4. **Pitch shift test**: Set camera.pitch to a non-zero value, verify star positions shift vertically. Set pitch to 0, verify no shift (regression guard).

## What We're NOT Doing

- No separate render passes or framebuffer layers
- No Z-buffer or depth sorting changes
- No new WorldState fields
- No changes to road, grid, sprites, or effects (they're already at parallax 1.0)
- No sun horizontal drift (pitch shift is sufficient for "nearly fixed")
