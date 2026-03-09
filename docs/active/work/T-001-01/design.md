# Design: T-001-01 bitmap-font

## Problem

`FONT_DATA` is all zeros. No text renders anywhere in git-demon despite working rendering logic.

## Approach Options

### Option A: Flat const array literal

Replace the const block with a plain `const FONT_DATA: [u8; 665] = [ ... ];` containing all 665 byte values inline.

- **Pros**: Simplest. No mutation, no const block tricks. Maximum compatibility. Easy to read with comments grouping glyphs.
- **Cons**: One large array literal. Harder to visually verify individual glyphs.

### Option B: Const block with mutable array and per-glyph initialization

Keep the const block pattern, use `let mut data = [0u8; 665];` then assign slices per glyph:
```rust
const FONT_DATA: [u8; 665] = {
    let mut d = [0u8; 665];
    // '!' = ASCII 33, offset 7
    d[7] = 0x20; d[8] = 0x20; ...
    d
};
```

- **Pros**: Each glyph assignment is self-contained and labeled. Easy to add/modify individual glyphs.
- **Cons**: Verbose (individual byte assignments). Const slice assignment (`d[off..off+7].copy_from_slice(...)`) requires nightly. Individual index assignment works on stable but is 7 lines per glyph × 95 glyphs = 665 assignment lines.

### Option C: Separate const arrays per glyph, assembled

Define each glyph as a named `const GLYPH_X: [u8; 7] = [...]` then assemble in a const block.

- **Pros**: Very readable per glyph.
- **Cons**: 95 separate const declarations plus a complex assembly const block. Overly engineered.

## Decision: Option A — Flat const array literal

Rationale:
1. The rendering code is already correct. The only change needed is populating data.
2. A flat array with comments per glyph block is the most straightforward representation.
3. It's the least code, most portable, and easiest to verify.
4. Comments can mark glyph boundaries: `// '0' (0x30)` before each 7-byte group.
5. The array is only 665 bytes — readable in a single screen page with compact formatting.

Option B was considered but the individual byte assignment syntax is more verbose without being more readable. Option C is over-engineered for static data.

## Font Data Source

Standard 5×7 bitmap font glyphs. The encoding is MSB-first, top-to-bottom, 7 bytes per glyph, bits 7–3 represent columns 0–4.

Each glyph will be hand-verified against the standard 5×7 pixel patterns. For example, 'A':
```
.#.#.   → 0x50? No...
```

Let me be precise. Column mapping:
- Bit 7 (0x80) = col 0
- Bit 6 (0x40) = col 1
- Bit 5 (0x20) = col 2
- Bit 4 (0x10) = col 3
- Bit 3 (0x08) = col 4

'A' in 5×7:
```
row 0: .#...  = 0x40? No...
```

Actually, typical 'A':
```
.###.   = 0x70
#...#   = 0x88
#...#   = 0x88
#####   = 0xF8
#...#   = 0x88
#...#   = 0x88
#...#   = 0x88
```

Wait — 5 columns means: `#` at positions 0–4.
```
.###.   bit7=0, bit6=1, bit5=1, bit4=1, bit3=0 → 0x70
#...#   bit7=1, bit6=0, bit5=0, bit4=0, bit3=1 → 0x88
#...#   → 0x88
#####   bit7=1, bit6=1, bit5=1, bit4=1, bit3=1 → 0xF8
#...#   → 0x88
#...#   → 0x88
#...#   → 0x88
```

This confirms the encoding. Each row byte uses bits 7–3.

## Test Strategy

Unit tests will:
1. Create a small `ImageBuffer` (e.g., 10×10)
2. Call `draw_char` for known glyphs ('A', '0', '!')
3. Check specific pixel positions match expected patterns
4. Test `text_width` returns correct values
5. Test `draw_text` positions characters correctly

## What Changes

- **`src/renderer/font.rs`**: Replace `FONT_DATA` initialization with complete 95-glyph data. No signature changes.
- **Tests**: Add `#[cfg(test)] mod tests` at bottom of `font.rs`.

## What Doesn't Change

- `draw_char`, `draw_text`, `text_width` function signatures and logic
- `GLYPH_W`, `GLYPH_H` constants
- All callers (hud.rs, sprites.rs)
- No new dependencies
