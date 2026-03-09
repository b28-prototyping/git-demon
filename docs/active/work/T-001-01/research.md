# Research: T-001-01 bitmap-font

## Current State

### File: `src/renderer/font.rs` (72 lines)

The module defines three public functions and one compile-time constant:

- **`FONT_DATA: [u8; 95 * 7]`** — Declared as a const block that returns `[0u8; 95 * 7]`. All 665 bytes are zero. No glyphs render.
- **`draw_char(fb, ch, x, y, color, scale)`** — Renders a single glyph from FONT_DATA into an `ImageBuffer<Rgba<u8>>`. Bounds-checks ASCII 32–126, computes glyph offset, iterates rows/cols with scale support.
- **`draw_text(fb, text, x, y, color, scale)`** — Iterates chars, calls `draw_char` with spacing of `(GLYPH_W + 1) * scale` = 6×scale pixels per character.
- **`text_width(text, scale)`** — Returns `count * 6 * scale - scale`. Subtracts one inter-char gap from the trailing edge.

### Bit Encoding

`draw_char` tests bits with `0x80 >> col` for col in 0..5. This means:
- Bit 7 (0x80) = column 0 (leftmost pixel)
- Bit 6 (0x40) = column 1
- Bit 5 (0x20) = column 2
- Bit 4 (0x10) = column 3
- Bit 3 (0x08) = column 4 (rightmost pixel)
- Bits 2–0 are unused

Each glyph is 7 consecutive bytes (one per row, top to bottom). Glyph index = `(ascii - 32) * 7`.

### Const Initialization Pattern

The current code uses:
```rust
const FONT_DATA: [u8; 95 * 7] = {
    let data = [0u8; 95 * 7];
    data
};
```
This is valid const Rust. To populate individual bytes in const context, we need a mutable array inside the const block (`let mut data = ...`) or a flat literal array.

### Callers

1. **`hud.rs`** — `draw_hud` renders 6 text elements: sector number, commits/min, lines added/deleted, files changed, tier name, repo name. Uses scale 1. Calls `text_width` for right-aligning repo name.

2. **`sprites.rs`** — `draw_sprites` renders text on:
   - `CommitBillboard` — commit message text, scale 1 or 2 depending on depth
   - `TierGate` — tier name (e.g., "CRUISE", "DEMON"), scale 1 or 2
   - `VelocitySign` — commits/min as "N.N" format, scale 1

### Character Coverage Required

From caller analysis:
- **Digits 0–9**: commits/min ("3.2"), lines ("+142"), files count, sector number, velocity sign
- **Uppercase A–Z**: tier names ("FLATLINE", "CRUISE", "ACTIVE", "DEMON", "VELOCITY DEMON"), "SECTOR"
- **Lowercase a–z**: repo name, commit messages, "c/min", "files", "repo"
- **Punctuation**: `.` (decimals), `/` (c/min), `+` `-` (lines), `:` (repo:), space
- **Additional useful**: `!`, `?`, `_`, `(`, `)`, `#`, `@`, `%`, `*`, `=`, `<`, `>`, `[`, `]`, `{`, `}`, `,`, `;`, `'`, `"`, `\`, `^`, `~`, `` ` ``, `|`, `&`, `$`

The acceptance criteria specify ASCII 32–126 (all 95 printable chars).

### Constants

- `GLYPH_W = 5`, `GLYPH_H = 7` — standard for minimal bitmap fonts
- Total data size: 95 × 7 = 665 bytes — trivially small, compile-time constant

### No External Dependencies

The module only imports `image::{ImageBuffer, Rgba}`. No file I/O, no runtime loading. The font data must remain a compile-time `const`.

### Testing

No tests exist for the font module currently. The acceptance criteria require unit tests verifying at least 3 glyphs render correct pixel patterns.

## Constraints

- Must remain `const` — no `lazy_static`, no `include_bytes!`, no file I/O
- Rust stable const eval supports mutable local arrays in const blocks (stabilized in Rust 1.83)
- The existing `draw_char`/`draw_text`/`text_width` signatures and behavior should be preserved (callers depend on them)
- Scale support (1× and 2×) already implemented in `draw_char` — no changes needed there

## Assumptions

- The 5×7 format is standard and well-documented; many open-source bitmap font references exist
- Space (ASCII 32) is correctly represented as all-zeros (already works)
- The bit encoding (MSB = leftmost) is correct and matches `draw_char` logic
