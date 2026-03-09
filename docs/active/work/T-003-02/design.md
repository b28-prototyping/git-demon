# T-003-02 Design: HUD Overlay

## Status

The HUD is already implemented. This design document evaluates the existing
implementation against acceptance criteria and identifies gaps to address.

## Acceptance Criteria Evaluation

| Criterion | Status | Notes |
|-----------|--------|-------|
| 18px strip at frame bottom | PASS | `HUD_HEIGHT = 18` |
| 80% alpha black background | NEAR | Alpha=200/255=78.4%, spec says 80% |
| Bitmap font at 1x scale | PASS | `font::draw_text(..., 1)` |
| SECTOR N field | PASS | `world.sector()` = `total_commits / 100` |
| X.X c/min field | PASS | `format!("{:.1} c/min", ...)` |
| +N -N field | PASS | `format!("+{}  -{}", ...)` |
| N files field | PASS | `format!("{} files", ...)` |
| Tier badge with colors | PASS | All 5 tiers with correct colors |
| VelocityDemon strobe 4Hz | PASS | `(world.time * 4.0) % 2` toggle |
| repo: name right-aligned | PASS | Uses `text_width()` for right-alignment |
| --no-hud hides HUD | PASS | Gated by `self.no_hud` in renderer |
| Alpha compositing | PASS | Manual per-pixel blend |

## Gaps Identified

### 1. Alpha value: 200 vs 204
The spec says 80% alpha. `0.80 * 255 = 204`. Current value is 200 (78.4%).
**Decision**: Fix to 204 for spec compliance. Visually imperceptible difference
but matches the stated requirement exactly.

### 2. No unit tests for HUD
`draw_hud()` and `tier_badge_color()` have zero test coverage.
**Decision**: Add tests covering:
- Background alpha compositing correctness
- Tier badge color values for each tier
- VelocityDemon strobe toggling
- Right-alignment of repo name
- Sector calculation

### 3. Hardcoded x-positions
Fields at fixed pixel offsets (8, 120, 240, 380, 480) will overlap on narrow
terminals. This is a known limitation but not an acceptance criterion violation.
**Decision**: Not addressing — the target use case (tmux corner pane) has
sufficient width, and dynamic layout adds complexity without clear benefit.

## Approach

Since the implementation is complete, the work is:
1. Fix alpha from 200 to 204
2. Add unit tests for `draw_hud()` and `tier_badge_color()`
3. Verify all acceptance criteria pass with tests

## Rejected Alternatives

### Dynamic field positioning
Could compute x-offsets based on text widths. Rejected because:
- Adds runtime computation every frame
- Current layout works for target screen sizes (>600px wide)
- Not required by acceptance criteria

### Separate HUD buffer
Could render HUD to a separate buffer and composite. Rejected because:
- Current direct-to-framebuffer approach is simpler and faster
- No intermediate allocation needed
- HUD is the last pass so no ordering issues
