#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

# Pull latest if this is an update
if [ -d .git ]; then
    if git remote get-url upstream &>/dev/null; then
        _BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
        [ -z "$_BRANCH" ] && _BRANCH="main"
        git pull --ff-only origin "$_BRANCH" 2>/dev/null || true
    else
        git pull --ff-only 2>/dev/null || true
    fi
fi

# Clean stale build artifacts
rm -f "$HOME/prj/util/bin/ink"
cargo clean

# Build
cargo build --release

# Install binary (crate is ink-md, binary defaults to ink-md; installed as ink)
mkdir -p "$HOME/prj/util/bin"
BIN_SRC="target/release/ink-md"
[ -f "$BIN_SRC" ] || BIN_SRC="target/release/ink"
cp "$BIN_SRC" "$HOME/prj/util/bin/ink"
chmod +x "$HOME/prj/util/bin/ink"

echo "Installed: $("$HOME/prj/util/bin/ink" --version 2>/dev/null || echo 'ink')"
