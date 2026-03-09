# T-004-01 Review: main-loop-integration

## Summary of Changes

**File modified:** `src/main.rs`

1. **Panic hook** (5 lines of production code): Added at top of `run()` before
   `ratatui::init()`. Calls `ratatui::restore()` on panic to prevent terminal
   from being left in raw mode, then delegates to the default panic handler so
   the panic message still prints normally.

2. **Test module** (~75 lines): 8 unit tests covering CLI argument parsing via
   `Args::try_parse_from()`:
   - Default value verification for all 11 fields
   - Flag parsing for --repo, --fps, --window, --interval, --render-fps, --scale
   - Boolean flag parsing for --no-blur, --no-bloom, --no-scanlines, --no-hud, --dev
   - Invalid flag rejection
   - dt clamp constant validation (0.05s matches spec)

## Acceptance Criteria Coverage

| Criterion | Status | Evidence |
|-----------|--------|---------|
| Picker::from_query_stdio() | Pass | Line 95 |
| Pixel dims from cell × terminal | Pass | compute_pixel_dims() line 67 |
| Git poller on background thread | Pass | GitPoller::spawn() line 92 |
| RepoSeed computed once | Pass | Line 94 |
| Loop: drain→update→render→display→input | Pass | Lines 125–176 |
| Frame timing: sleep remaining | Pass | Lines 179–181 |
| dt clamped at 0.05s | Pass | Line 126, test_dt_clamp_constant |
| 'q' and Esc exit | Pass | Line 170 |
| Resize triggers reallocation | Pass | Lines 171–174 |
| ratatui::restore() on normal+error | Pass | Lines 184, 191, + panic hook |
| CLI args via clap | Pass | Lines 15–65, 8 tests |

## Test Coverage

- **Total tests:** 164 (156 lib + 8 bin), all passing
- **New tests:** 8 in main.rs covering CLI argument parsing
- **Coverage gaps:**
  - `compute_pixel_dims()` is not directly unit tested because it requires a
    real `Picker` and `Terminal`. The math is trivial (multiply + clamp).
  - The main loop itself is not tested (requires real terminal + git repo).
    This is acceptable — the loop is pure orchestration of well-tested
    components.
  - Panic hook behavior cannot be automatically tested without spawning a
    subprocess.

## Open Concerns

1. **Pre-existing: framebuffer clone per render frame.** `fb.clone()` on line
   158 allocates ~7MB per render at full resolution. This violates the spec's
   "zero heap allocation in render hot path" but is inherent to the
   ratatui-image API requiring an owned `DynamicImage`. Not in this ticket's
   scope.

2. **Pre-existing: unused assignment warning.** Line 111 assigns to `proto`
   but the value is overwritten before being read. Harmless but noisy. Could
   be addressed by initializing `proto` lazily or using `MaybeUninit`, but
   that adds complexity for no functional benefit.

3. **FPS defaults differ from spec.** Spec says `--fps 60` default; code uses
   30 with a separate `--render-fps 15`. This dual-rate design was an
   intentional performance decision made during earlier tickets and is the
   correct approach.

## No Files Created or Deleted

Only `src/main.rs` was modified. No new modules, no dependency changes.
