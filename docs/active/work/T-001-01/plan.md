# Plan: T-001-01 bitmap-font

## Step 1: Populate FONT_DATA with all 95 glyphs

Replace the zero-initialized const block with a flat array literal containing correct 5×7 bitmap data for ASCII 32–126.

Each glyph is 7 bytes using MSB-first encoding (bits 7–3 = columns 0–4).

Glyphs to include (all 95 printable ASCII):
- Space (32): all zeros
- Punctuation (33–47): ! " # $ % & ' ( ) * + , - . /
- Digits (48–57): 0–9
- Punctuation (58–64): : ; < = > ? @
- Uppercase (65–90): A–Z
- Punctuation (91–96): [ \ ] ^ _ `
- Lowercase (97–122): a–z
- Punctuation (123–126): { | } ~

Verification: each glyph row byte should produce the expected pixel pattern when bits 7–3 are tested.

## Step 2: Add unit tests

Add `#[cfg(test)] mod tests` to the bottom of `font.rs` with:

1. **`test_glyph_A`** — Call `draw_char` for 'A' at scale 1 on a 5×7 buffer. Verify row 0 has pixels at (1,0),(2,0),(3,0) lit (the `.###.` pattern), row 3 has all 5 lit (`#####`), etc.

2. **`test_glyph_0`** — Call `draw_char` for '0'. Verify the oval shape: top/bottom rows have middle 3 pixels, sides have outer 2 pixels.

3. **`test_glyph_exclamation`** — Call `draw_char` for '!'. Verify center column lit for rows 0–3, blank row 4, center lit row 5.

4. **`test_text_width`** — Verify `text_width("AB", 1) == 11` (5+1+5), `text_width("A", 2) == 10` (5*2), `text_width("", 1) == 0`.

5. **`test_draw_text_spacing`** — Render "AB" and verify 'B' starts at x=6 (at scale 1).

6. **`test_space_is_blank`** — Render space, verify no pixels are set.

## Step 3: Verify

- `cargo test` — all new tests pass
- `cargo clippy` — no warnings
- `cargo build` — compiles cleanly

## Testing Strategy

All tests are unit tests within `font.rs`. They create small `ImageBuffer` instances, render glyphs, and check pixel values. No integration tests needed — the font is pure data + pure rendering logic.

Test verification approach: check that specific pixels are set (non-zero alpha) or unset (zero alpha / not white) after rendering with a known color onto a black background.
