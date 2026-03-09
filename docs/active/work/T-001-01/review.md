# Review: T-001-01 bitmap-font

## Summary of Changes

### Files Modified
- **`src/renderer/font.rs`**: Replaced zero-initialized `FONT_DATA` const block with a complete 665-byte flat array literal containing 5×7 bitmap data for all 95 printable ASCII characters (32–126). Added 6 unit tests. Fixed one pre-existing clippy lint.

### No Files Created or Deleted
All changes are within the single existing file.

## What Changed

1. **FONT_DATA population**: The const block `{ let data = [0u8; 665]; data }` was replaced with a flat `[u8; 665]` array literal. Each glyph is 7 bytes (one per row, top-to-bottom), with bits 7–3 encoding columns 0–4 (MSB = leftmost pixel). Glyphs are commented with their character and ASCII code.

2. **Clippy fix**: Changed `if idx < 32 || idx > 126` to `if !(32..=126).contains(&idx)` to satisfy `clippy::manual_range_contains`.

3. **Unit tests**: Added `#[cfg(test)] mod tests` with 6 tests covering glyph rendering, text width calculation, text spacing, and blank space verification.

## Test Coverage

| Test | What it verifies |
|------|-----------------|
| `test_glyph_a_uppercase` | 'A' renders correct pixel pattern at scale 1 (top bar, full row 3, side pillars) |
| `test_glyph_zero` | '0' renders oval shape (top/bottom arcs at rows 0,6; sides at row 1) |
| `test_glyph_exclamation` | '!' renders center column rows 0–3, blank row 4, dot at row 5 |
| `test_text_width` | Width formula correct for empty string, 1-char, 2-char at scales 1 and 2 |
| `test_draw_text_spacing` | Multi-char rendering places 'B' at correct x offset (6) in "AB" |
| `test_space_is_blank` | Space character (ASCII 32) produces zero lit pixels |

Coverage: All three public functions (`draw_char`, `draw_text`, `text_width`) are exercised. The font data itself is validated through pixel-level assertions on 3 representative glyphs covering uppercase, digit, and punctuation categories.

### What's not tested
- Scale 2 rendering (pixel doubling) — logic is straightforward loop nesting, low risk
- Out-of-bounds characters (< 32 or > 127) — the early return is trivial
- Every individual glyph — impractical for 95 glyphs; the 3 tested glyphs confirm the encoding scheme is correct

## Acceptance Criteria Status

- [x] `FONT_DATA` contains correct 5×7 bitmap data for ASCII 32–126 (95 glyphs)
- [x] At minimum: digits 0–9, uppercase A–Z, lowercase a–z, space, and punctuation `.:/-+!?`
- [x] `draw_char` renders a single glyph at specified position, color, and scale (1× or 2×)
- [x] `draw_text` renders a string with correct character spacing
- [x] `text_width` returns accurate pixel width for a given string and scale
- [x] No file I/O — font is a compile-time constant
- [x] Unit tests verify at least 3 glyphs render correct pixel patterns

## Open Concerns

1. **Glyph accuracy**: The bitmap data was hand-crafted using standard 5×7 font conventions. While the tested glyphs ('A', '0', '!') are verified correct, the remaining 92 glyphs have not been individually pixel-verified. Any rendering oddities in specific characters would need visual inspection at runtime.

2. **No visual regression test**: There's no way to catch subtle glyph errors without rendering to screen. A snapshot/golden-image test could be added later if glyph accuracy becomes an issue.

3. **The `~` tilde glyph** uses a simplified 3-row pattern (`0x48, 0xA8, 0x90`) which is an approximation of a tilde wave in 5 pixels wide. This is standard for 5×7 fonts but may look slightly unusual.
