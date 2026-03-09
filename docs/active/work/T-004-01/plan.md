# T-004-01 Plan: main-loop-integration

## Step 1: Add panic hook

**Change:** In `run()`, before `ratatui::init()`, add a panic hook that calls
`ratatui::restore()` before delegating to the default panic handler.

**Code:**
```rust
let default_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    default_hook(info);
}));
```

Note: We use crossterm directly instead of `ratatui::restore()` because the
ratatui restore function might not be safe to call from a panic context (it
could panic itself). The crossterm calls are lower-level and more defensive.

Actually, `ratatui::restore()` is just:
```rust
pub fn restore() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
}
```

It already uses `let _ =` to ignore errors. It's safe in panic context. Use it.

**Verification:** `cargo build` succeeds. Manual test: add a temporary
`panic!()` in the loop and verify terminal restores properly. (Not automated.)

## Step 2: Add CLI argument tests

**Change:** Add `#[cfg(test)] mod tests` at bottom of `main.rs` with tests for
`Args::try_parse_from()`.

**Tests:**

1. `test_args_defaults` — parse with just `["git-demon"]`, verify all defaults:
   - repo = "."
   - window = 30
   - interval = 30
   - fps = 30
   - scale = 0.5
   - render_fps = 15
   - no_blur = false, no_bloom = false, no_scanlines = false, no_hud = false
   - dev = false

2. `test_args_repo_flag` — parse `["git-demon", "--repo", "/tmp/foo"]`,
   verify repo = "/tmp/foo"

3. `test_args_all_disable_flags` — parse with all --no-* flags, verify all true

4. `test_args_numeric_overrides` — parse with --fps 60 --window 10 --interval 5,
   verify values

5. `test_args_scale_override` — parse with --scale 1.0, verify

6. `test_args_dev_flag` — parse with --dev, verify true

7. `test_args_invalid_rejects` — parse with unknown flag, verify error

## Step 3: Run test suite

**Verification:**
- `cargo test` — all tests pass
- `cargo clippy` — no warnings
- `cargo build` — clean build

## Testing Strategy

- **Unit tests:** CLI arg parsing via `Args::try_parse_from()` — no external
  dependencies, fast, deterministic
- **No integration tests:** The main loop requires a real terminal and git
  repo. Integration testing would require mocking terminal I/O, which is
  brittle and low-value for a screensaver
- **Manual verification:** Run `cargo run -- --repo . --dev` and verify all
  components work together (already done during development)

## Commit Plan

Single commit with panic hook + tests, since they're a cohesive unit of work.
