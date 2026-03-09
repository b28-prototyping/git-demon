# T-004-02 Benchmark Results

## Machine
- Platform: macOS (Darwin 25.3.0)
- Profile: `bench` (optimized release build)
- Criterion 0.8, plotters backend

## Full Pipeline Results (1920x960)

| Benchmark | Median | Target | Status |
|---|---|---|---|
| all_effects | 4.66 ms | <4 ms | ~17% over |
| no_effects | 1.97 ms | — | baseline |
| velocity_demon | 4.42 ms | <4 ms | ~10% over |

## Individual Pass Breakdown

| Pass | Median | % of full |
|---|---|---|
| sky (gradient + sun + bloom bleed) | 0.285 ms | 6.1% |
| road (scanlines + grid) | 1.524 ms | 32.7% |
| terrain (simplex noise + trees) | 0.056 ms | 1.2% |
| sprites (12 objects) | 0.007 ms | 0.1% |
| effects (blur + scanline + bloom) | 2.183 ms | 46.8% |

Sum of passes: ~4.05 ms. Remaining ~0.6 ms is world.update(), orchestration overhead, and buffer swap in render().

## Analysis

The two dominant passes are:
1. **Effects** (2.18 ms / 46.8%) — motion blur processes entire 1920x960 buffer in raw byte operations, plus scanline filter and bloom with emissive pixel scan
2. **Road** (1.52 ms / 32.7%) — per-pixel scanline rasterizer iterates every pixel in the bottom 65% of the frame

Together these account for ~80% of render time. Sky, terrain, and sprites are negligible.

The full pipeline at 4.66 ms is above the 4 ms target. The no-effects baseline at 1.97 ms shows the rasterizer core is well within budget — the overhead is in post-processing effects.

VelocityDemon (4.42 ms) is actually slightly faster than all_effects Demon tier (4.66 ms), likely due to measurement variance and the fact that both share the same effects pipeline — the VelocityDemon horizon/road adjustments have minimal cost.

## Optimization Opportunities (out of scope for this ticket)
- **Motion blur**: Could use SIMD (packed u8 operations) for ~2-4x speedup on the full-buffer blend
- **Road rasterizer**: Inner loop does per-pixel branch for stripe/rumble/verge — could precompute column ranges per row
- **Bloom**: Emissive pixel scan iterates buffer at 2x stride — could skip entirely when no emissive pixels
- **Scanline filter**: Already uses fixed-point math — could batch with SIMD
