# Watch for changes, rebuild, and run git-demon with dev overlay
dev:
    cargo watch -w src -w Cargo.toml --postpone -x 'run -- --repo . --fps 30 --no-bloom --no-blur --dev'

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
