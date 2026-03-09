# T-004-01 Structure: main-loop-integration

## Files Modified

### src/main.rs

**Additions:**

1. Panic hook — added at the top of `run()`, before `ratatui::init()`:
   - `std::panic::take_hook()` to preserve default handler
   - `std::panic::set_hook()` with closure that calls `ratatui::restore()`
     then delegates to default hook

2. `#[cfg(test)] mod tests` block at bottom of file:
   - CLI argument parsing tests using `Args::try_parse_from()`
   - Tests verify default values match expected behavior
   - Tests verify flag parsing (--no-blur, --no-bloom, etc.)
   - Tests verify --repo path parsing

**No structural changes to existing code.** The loop, imports, and function
signatures remain identical. The only production code change is ~5 lines for
the panic hook.

## Module Boundaries

No new modules. No new files. No changes to public interfaces.

The panic hook is internal to `run()` — it's a setup concern, not an
architectural change.

## Component Interactions

```
main()
  └── run(args)
        ├── [NEW] set panic hook (restore terminal)
        ├── ratatui::init()
        ├── Picker::from_query_stdio()
        ├── GitPoller::spawn()
        ├── RepoSeed::compute()
        ├── loop { ... }  (unchanged)
        └── ratatui::restore()
```

## Test Module Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // CLI default values
    fn test_args_defaults()
    fn test_args_repo_flag()
    fn test_args_window_flag()
    fn test_args_interval_flag()
    fn test_args_fps_flag()
    fn test_args_scale_flag()
    fn test_args_render_fps_flag()
    fn test_args_disable_flags()
    fn test_args_dev_flag()
}
```

## Files NOT Modified

- `src/lib.rs` — no public API changes
- `src/renderer/*` — rendering unchanged
- `src/world/*` — simulation unchanged
- `src/git/*` — polling unchanged
- `Cargo.toml` — no new dependencies

## Ordering

1. Add panic hook (production code)
2. Add test module (test code)
3. Verify `cargo test` passes
4. Verify `cargo clippy` clean
