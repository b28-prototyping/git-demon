# Design — T-002-01: repo-seed-computation

## Decisions

### 1. accent_hue: Use origin URL, fall back to first commit SHA

**Chosen approach**: Hash the remote origin URL string to derive `accent_hue`. If no
remote exists, fall back to the root commit SHA (last commit in time-sorted revwalk).
If no commits either (empty repo), fall back to hashing the repo directory name.

**Rationale**: The AC explicitly states "derived from remote origin URL (or first commit
SHA if no remote)." Using the origin URL means forks/clones of the same repo get the
same visual identity. The current code uses HEAD SHA, which changes with every new
commit — it's only stable because `compute()` runs once at startup, but conceptually
it's wrong for identity.

**Note on "first commit SHA"**: The AC says "first commit SHA" meaning the repo's root
commit (earliest), not HEAD. This is more stable — it never changes. In a time-sorted
revwalk, the root commit is the *last* one encountered. We'll capture it during the walk.

**Rejected**: Using HEAD SHA (current behavior) — changes every commit, wrong identity.

### 2. terrain_roughness: Variance of inter-commit intervals

**Chosen approach**: Collect commit timestamps during the revwalk. Compute inter-commit
intervals in seconds. Calculate the coefficient of variation (stddev / mean) of these
intervals, then map to `[0.1, 1.0]`.

- Repos with regular cadence → low variance → smoother terrain
- Repos with bursty activity → high variance → rougher terrain

**Formula**:
```
intervals = [t[i] - t[i+1] for consecutive commits, time-sorted descending]
mean = avg(intervals)
stddev = sqrt(avg((interval - mean)²))
cv = stddev / mean  // coefficient of variation, dimensionless
terrain_roughness = (cv * 0.5).clamp(0.1, 1.0)
```

The 0.5 scaling factor keeps typical repos in a useful range. A CV of 2.0 (very bursty)
maps to 1.0. A CV of 0.2 (very regular) maps to 0.1.

**Edge cases**:
- 0-1 commits → no intervals → default 0.1
- All same timestamp → mean=0, CV undefined → default 0.1

**Rejected alternatives**:
- Keep log10(total_commits): Doesn't measure variance, just size. Violates AC.
- Per-week commit count variance: Requires binning into calendar weeks, more complex
  for marginal benefit over raw interval variance.

### 3. Empty repo handling

**Chosen approach**: Check if `push_head()` succeeds. If it fails (empty repo or
unborn branch), skip the revwalk entirely and use defaults:
- `accent_hue` = hash of origin URL or directory name
- `terrain_roughness` = 0.1
- `speed_base` = 0.0
- `author_colors` = empty HashMap
- `total_commits` = 0

**Rationale**: `push_head()` returns `Err` when HEAD doesn't point to a valid commit.
We catch this error instead of propagating it.

### 4. Unit tests for HSL→RGB

**Chosen approach**: Add `#[cfg(test)] mod tests` at the bottom of seed.rs with:
- Known HSL→RGB conversions (red, green, blue, white, black, gray)
- `hash_to_hue` determinism test (same input → same output)
- `hash_to_hue` range test (output always in [0, 360))
- Edge case: empty string hash

Tests use the private functions directly since `mod tests` is inside the module.

### 5. Error type

**Keep `git2::Error`** as the return type. The only fallible operations are git2 calls.
No need to change the API surface.

## Non-goals

- Deduplicating HSL→RGB across modules (separate concern, not in AC)
- Changing the `saturation` field or its fixed 0.8 value
- Modifying how consumers use `RepoSeed`
