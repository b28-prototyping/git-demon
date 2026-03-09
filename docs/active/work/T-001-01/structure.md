# Structure: T-001-01 bitmap-font

## Files Modified

### `src/renderer/font.rs`

**Change**: Replace FONT_DATA zero-initialization with complete 95-glyph bitmap data.

**Before** (lines 9–17):
```rust
const FONT_DATA: [u8; 95 * 7] = {
    let data = [0u8; 95 * 7];
    data
};
```

**After**: A flat `#[rustfmt::skip]` const array literal with all 665 bytes, organized in commented groups:
```rust
#[rustfmt::skip]
const FONT_DATA: [u8; 95 * 7] = [
    // ' ' (32) - space
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // '!' (33)
    0x20, 0x20, 0x20, 0x20, 0x00, 0x20, 0x00,
    // '"' (34)
    ...
    // '~' (126)
    ...
];
```

**Addition**: Test module appended at end of file:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    // Test helpers and glyph verification tests
}
```

## Module Boundaries

No changes to module boundaries. `font.rs` remains a leaf module with no internal submodules. Its public API stays identical:

- `pub fn draw_char(fb, ch, x, y, color, scale)`
- `pub fn draw_text(fb, text, x, y, color, scale)`
- `pub fn text_width(text, scale) -> u32`

## No New Files

All changes are within `src/renderer/font.rs`. No new modules, no new dependencies.

## Encoding Reference

Each glyph = 7 bytes, one per row (top to bottom).
Each byte = 5 pixel columns packed into bits 7–3 (MSB = leftmost).

Glyph offset for ASCII code `c` = `(c - 32) * 7`.

Character ranges:
- Offset 0–6: space (32)
- Offset 7–13: ! (33)
- ...
- Offset 658–664: ~ (126)

## Ordering of Changes

1. Replace FONT_DATA (the bulk of the work — 665 bytes of glyph data)
2. Add test module
3. Verify with `cargo test` and `cargo clippy`

No ordering dependencies with other files or tickets.
