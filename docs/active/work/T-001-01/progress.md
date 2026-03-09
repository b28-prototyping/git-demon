# Progress: T-001-01 bitmap-font

## Completed

### Step 1: Populate FONT_DATA with all 95 glyphs
- Replaced zero-initialized const block with flat `#[rustfmt::skip]` array literal
- All 95 printable ASCII characters (32–126) encoded as 7 bytes each, MSB-first
- Each glyph group commented with character and ASCII code
- Total: 665 bytes of bitmap data

### Step 2: Add unit tests
- Added `#[cfg(test)] mod tests` with 6 tests:
  - `test_glyph_a_uppercase` — verifies 'A' pixel pattern (top bar, full row, side pillars)
  - `test_glyph_zero` — verifies '0' oval shape (top/bottom arcs, sides)
  - `test_glyph_exclamation` — verifies '!' column + dot pattern
  - `test_text_width` — verifies width calculation for empty, single, multi-char at scales 1 and 2
  - `test_draw_text_spacing` — verifies 'B' starts at x=6 when rendering "AB"
  - `test_space_is_blank` — verifies space glyph produces no lit pixels

### Step 3: Verify
- `cargo test` — all 14 tests pass (6 new font tests + 8 existing)
- `cargo clippy` — fixed one pre-existing `manual_range_contains` warning in `draw_char`
- `cargo build` — compiles cleanly

## Deviations from Plan

- Fixed a pre-existing clippy warning (`manual_range_contains`) in `draw_char` while working in the file. This was not in the plan but is a trivial improvement and eliminates a clippy warning.

## Remaining

Nothing. All steps complete.
