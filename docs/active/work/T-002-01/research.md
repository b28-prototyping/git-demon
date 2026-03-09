# Research — T-002-01: repo-seed-computation

## Current State

`RepoSeed` is fully implemented in `src/git/seed.rs` (120 lines). The struct and its
`compute()` method exist and are integrated into the application.

### Struct Fields (seed.rs:4-12)

```rust
pub struct RepoSeed {
    pub accent_hue: f32,              // 0-360
    pub saturation: f32,              // fixed 0.8
    pub terrain_roughness: f32,       // 0.1-1.0
    pub speed_base: f32,              // 0-1.0
    pub author_colors: HashMap<String, Rgba<u8>>,
    pub total_commits: u64,
    pub repo_name: String,
}
```

### Current compute() Flow (seed.rs:15-90)

1. Opens repo via `git2::Repository::open(repo_path)`
2. Derives `repo_name` from remote "origin" URL basename (strips `.git`) or dir name
3. Walks history via `revwalk()` with `Sort::TIME`, `push_head()`
4. Captures first OID (HEAD) as identity hash, counts commits, tallies authors
5. `accent_hue = hash_to_hue(&first_sha)` — djb2 hash mod 360
6. Author colors: `hash_to_hue(name)` → `hsl_to_rgb(h, 0.8, 0.6)` → `Rgba`
7. `terrain_roughness = log10(total_commits) / 5.0`, clamped `[0.1, 1.0]`
8. `speed_base = (total_commits / 1000.0).min(1.0)`

### Helper Functions (seed.rs:93-119)

- `hash_to_hue(s: &str) -> f32`: djb2 variant, mod 360
- `hsl_to_rgb(h, s, l) -> (u8, u8, u8)`: standard 6-sector HSL conversion

Both are private (`fn`, not `pub fn`).

## Consumers

| File | Usage |
|------|-------|
| `main.rs:69` | `RepoSeed::compute(&args.repo)?` — one-time at startup |
| `main.rs:73` | `WorldState::new(&seed)` — seeds simulation |
| `main.rs:86` | `world.ingest_poll(&result, &seed)` — author color lookup |
| `main.rs:91` | `renderer.render(&world, &seed)` — accent hue, roughness |
| `world/mod.rs:37-42` | `speed_target = 1.5 + seed.speed_base * 2.8` |
| `world/objects.rs:55-89` | `seed.author_colors.get(&commit.author)` |
| `renderer/sky.rs` | `seed.accent_hue` for gradient + complement for sun |
| `renderer/road.rs` | `seed.accent_hue` for neon grid |
| `renderer/terrain.rs` | `seed.terrain_roughness` for height scaling |
| `renderer/hud.rs` | `seed.repo_name` display |

## Gaps vs. Acceptance Criteria

### 1. accent_hue derivation (AC says: origin URL, fallback first commit SHA)
**Current**: Uses HEAD SHA (`first_sha`), never considers remote URL.
**Required**: Hash remote origin URL first; only fall back to first commit SHA if no remote.

### 2. terrain_roughness (AC says: "commit frequency variance")
**Current**: `log10(total_commits) / 5.0` — based on total count, not variance.
**Required**: Derive from variance of commit frequency (e.g., per-week or per-day variance).
This needs commit timestamps to compute intervals.

### 3. Empty repo edge case (AC: "Handles edge cases: empty repo (0 commits)")
**Current**: `revwalk.push_head()?` propagates an error on empty repos (no HEAD).
**Required**: Graceful fallback when HEAD doesn't exist.

### 4. Detached HEAD edge case
**Current**: `push_head()` works fine with detached HEAD — it pushes the OID HEAD
points to, regardless of branch state. No issue here.

### 5. HSL→RGB unit tests (AC: "HSL→RGB color conversion is correct (unit tested)")
**Current**: No tests in `seed.rs`. Similar HSL functions in sky.rs and road.rs have
tests, but seed.rs's copy does not.
**Required**: Unit tests for `hsl_to_rgb` and `hash_to_hue` in seed.rs.

### 6. Duplicate HSL→RGB implementations
Three copies of `hsl_to_rgb` exist: seed.rs:101, sky.rs, road.rs.
Not in the AC but worth noting — deduplication is out of scope for this ticket.

## Constraints

- `RepoSeed` is borrowed immutably (`&seed`) throughout the app lifetime — no mutation
- `compute()` returns `Result<Self, git2::Error>` — changing to `anyhow::Error` would
  require changing the call site in main.rs
- `hash_to_hue` and `hsl_to_rgb` are private — tests need to be in the same module
  or the functions need `pub(crate)` visibility
- The revwalk collects commit timestamps already (`commit.time()`) — we can add
  interval variance computation without an extra pass

## Dependencies

- `git2 0.20`: `Repository::open`, `Remote::url`, `Revwalk`, `Commit`
- `image 0.25`: `Rgba<u8>` type
- `std::collections::HashMap`
