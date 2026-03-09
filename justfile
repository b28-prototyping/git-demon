# Watch for changes, rebuild, and run git-demon with dev overlay
# First Ctrl+C stops the program; second Ctrl+C exits cargo watch.
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    cleanup() { reset 2>/dev/null; stty sane 2>/dev/null; }
    trap cleanup EXIT
    cargo watch -w src -w Cargo.toml -s 'cargo run -- --repo . --fps 30 --render-fps 30 --scale 1 --no-bloom --no-blur --dev; reset 2>/dev/null; stty sane 2>/dev/null'

# Run in release mode, full resolution, max performance
start *ARGS:
    cargo run --release -- --repo . --fps 30 --render-fps 15 --scale 1 {{ARGS}}

# Single run without watch
run *ARGS:
    cargo run -- --repo . {{ARGS}}

# Commit, test, push, and file bugs for failures
commit-agent:
    @just _commit-agent

# ---------- commit-agent ----------

[private]
_commit-agent:
    #!/usr/bin/env bash
    set -euo pipefail
    exec claude \
      --allowedTools 'Read,Write,Glob,Grep,Bash(git status*),Bash(git diff*),Bash(git log*),Bash(git add *),Bash(git commit *),Bash(git push *),Bash(lisa status),Bash(lisa validate),Bash(cargo test*),Bash(cargo fmt*),Bash(cargo clippy*),Edit' \
      -- "$(cat .just/prompts/commit-agent.md)"
