# T-004-01 Design: main-loop-integration

## Problem Summary

The main loop is already functional. The remaining work is:
1. Add a panic hook to restore terminal state on panics
2. Add unit tests for the testable pure functions in main.rs
3. Verify all acceptance criteria pass

## Option A: Minimal — Panic Hook + Tests Only

Add `std::panic::set_hook()` before entering the main loop to call
`ratatui::restore()` on panic. Add unit tests for `compute_pixel_dims()` and
CLI arg parsing. No structural changes.

**Pros:**
- Smallest diff, lowest risk
- Fixes the one real gap (terminal restore on panic)
- Tests cover the only testable surface

**Cons:**
- Doesn't address framebuffer clone allocation (but that's out of scope)

## Option B: Refactor run() for Testability

Extract loop body into smaller functions, add a trait for terminal interaction
to enable mock-based testing of the loop itself.

**Pros:**
- More test coverage of the orchestration logic

**Cons:**
- Over-engineering for a screensaver
- Mock-based terminal tests are fragile and low-value
- Significant refactor for minimal gain
- The loop is 60 lines — it doesn't need decomposition

## Option C: Drop Guard Pattern

Replace the panic hook with a RAII drop guard that calls `ratatui::restore()`
when dropped, covering both normal exit, errors, and panics.

**Pros:**
- Cleaner than a global panic hook
- Covers all exit paths by construction

**Cons:**
- `ratatui::restore()` is safe to call multiple times, so double-restore is
  fine, but the guard needs to live in `run()` scope
- Slightly more code than a simple panic hook
- `ratatui::init()` returns the terminal, and `ratatui::restore()` is a free
  function — a guard that just calls `restore()` on drop is trivial

## Decision: Option A + elements of Option C

Use a panic hook (simplest approach) to call `ratatui::restore()` on panic.
The drop guard pattern (Option C) is slightly cleaner but adds a struct for
a one-liner — the panic hook is more direct.

Add tests for:
- `compute_pixel_dims()` with various inputs
- CLI arg parsing via `Args::try_parse_from()`
- Verify dt clamp constant matches spec (0.05s)

Do NOT:
- Refactor the loop structure (it's already clean)
- Change the dual-rate fps design (it's intentional and better than spec)
- Address framebuffer clone (separate performance concern)
- Add integration tests requiring terminal mocking

## Panic Hook Design

```rust
let default_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    ratatui::restore();
    default_hook(info);
}));
```

Place this at the top of `run()`, before `ratatui::init()`. This ensures the
terminal is restored even if a panic occurs deep in the render pipeline. The
default hook is preserved so the panic message still prints.

## Test Design

Tests go in a `#[cfg(test)] mod tests` block at the bottom of `main.rs`.

1. **compute_pixel_dims tests:** Create a mock Picker is not feasible (no
   public constructor for test). Instead, test the math inline: verify that
   the formula `(cols * cell_w * scale).max(160)` produces expected values
   for known inputs. Since `compute_pixel_dims` requires a real Picker and
   Terminal, we'll extract the math into a helper or test the contract
   indirectly.

   Actually, looking at the code again: `compute_pixel_dims` calls
   `picker.font_size()` and `terminal.size()` — both require real objects.
   The pure math is trivial (multiply and clamp). Not worth testing.

2. **CLI args:** Use `Args::try_parse_from()` to verify defaults and flag
   parsing. This is testable without any external state.

3. **dt clamp:** Verify the constant in the code matches spec (0.05s = 20fps
   minimum). This is a code-level assertion, not really a test.

## Changes Summary

| File | Change |
|------|--------|
| src/main.rs | Add panic hook in run(), add #[cfg(test)] mod tests |
