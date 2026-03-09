# T-005-01 Design: fix-grid-line-scrolling

## Problem

Grid lines are static because `z_world = i * grid_spacing` never incorporates
camera movement. They need to scroll toward the camera and wrap around.

## Options

### Option A: Offset with `camera_z % grid_spacing`

```rust
let camera_offset = world.camera_z % grid_spacing;
let z_world = i as f32 * grid_spacing - camera_offset;
if z_world <= 0.0 { continue; }
```

The modulo creates a repeating cycle: as `camera_z` advances through one
`grid_spacing` interval, all lines shift toward the camera by that amount.
When a line crosses z=0, it wraps back to the far end.

Pros:
- Minimal change (3 lines modified)
- Matches the ticket's suggested fix exactly
- Uses `camera_z` which semantically represents camera position
- No new state, no new fields, no allocations

Cons:
- None significant

### Option B: Offset with `z_offset % grid_spacing`

Same approach but using `z_offset` instead of `camera_z`.

Pros:
- `z_offset` is what road stripes use, so conceptually aligned

Cons:
- `z_offset` and `camera_z` are identical in practice (both += speed*dt)
- `camera_z` is more semantically correct for "where the camera is"
- Ticket explicitly suggests `camera_z`

### Option C: Absolute world positions offset by camera_z

Instead of subtracting from grid positions, compute grid positions relative
to camera:

```rust
let base = (world.camera_z / grid_spacing).floor() * grid_spacing;
for i in 1..40 {
    let z_world = base + i as f32 * grid_spacing - world.camera_z;
}
```

Pros:
- Makes the camera-relative nature explicit

Cons:
- More complex for same result
- The floor/multiply dance is equivalent to modulo

## Decision: Option A

Option A is the simplest correct solution. It's the exact fix described in the
ticket, requires minimal code change, and directly addresses the root cause.
The `camera_z % grid_spacing` approach is a standard technique for scrolling
tiled/repeated elements.

The `z_world <= 0.0` guard prevents drawing lines behind the camera. This also
ensures the line count stays roughly constant — as one line scrolls past z=0
and gets skipped, the iteration range naturally covers a new line at the far end.

## Edge Cases

1. **camera_z = 0**: `camera_offset = 0`, identical to current behavior. Correct.
2. **camera_z negative**: Won't happen in practice (camera only advances), but
   `f32 % positive` handles it correctly in Rust (result has same sign as dividend).
   The `z_world <= 0.0` guard still works.
3. **Very large camera_z**: `f32 % 5.0` is precise even at large values. No issue.
4. **grid_spacing boundary**: When `camera_z` is exactly a multiple of
   `grid_spacing`, `camera_offset = 0` — clean state, no discontinuity.

## Visual Behavior

- Lines scroll toward camera at road speed
- When a line reaches z=0, it disappears and a new one appears at the far end
- The number of visible lines stays constant (determined by draw_distance / grid_spacing)
- Lines still respect road curvature (curve_shift applied independently)
- No change to alpha blending, color, or spacing
