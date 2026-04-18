# Security Review — ink

Reviewed upstream: https://github.com/borghei/ink (branch `main` @ 01297c5)
Date: 2026-04-18

## Summary

Code audit: LOW risk. Dependency audit: HIGH risk (2 rustls-webpki CVEs, now
patched in this fork).

## Code Scan (LOW)

Rust terminal markdown reader. No unsafe blocks, no `build.rs`, no
`std::process::Command` invocations, no credential access (`~/.ssh`, `~/.aws`,
etc.), no TLS verification disabled. TLS is `rustls-tls` only. Upstream
`install.sh` uses HTTPS with `set -e` but downloads release binaries without
SHA256 verification — not used by this local install.

Notes:
- `open = "5"` crate is declared in `Cargo.toml` but not actually called in
  `src/` (link opening is via OSC-8 escapes). Unused dep, kept for upstream
  parity.
- `src/main.rs:294` fetches user-supplied URLs via `reqwest::blocking::get`
  with no timeout or size cap (image loader in `src/image.rs` uses a 10s
  timeout). User-directed fetch, lower priority.
- `install.sh` (upstream prebuilt-binary installer) is not used here; we
  build from source.

## Dependency Audit

`cargo audit` against 356 crates (RustSec DB with 1049 advisories).

### Patched in this fork

- `rustls-webpki 0.103.10` → `0.103.12`
  - RUSTSEC-2026-0098: Name constraints for URI names were incorrectly accepted
  - RUSTSEC-2026-0099: Name constraints were accepted for certificates asserting a wildcard name
  - Fix: `cargo update -p rustls-webpki` (Cargo.lock bumped).

### Outstanding (transitive, no local fix)

- `rand 0.8.5` / `rand 0.9.2` — RUSTSEC-2026-0097 (unsound; transitive via
  `ratatui-termwiz`/`phf` and `quinn-proto`).
- `bincode 1.3.3` — RUSTSEC-2025-0141 (unmaintained; transitive via
  `syntect`/`comrak`).
- `yaml-rust 0.4.5` — RUSTSEC-2024-0320 (unmaintained; transitive via
  `syntect`).

Requires upstream `syntect`/`quinn` releases to resolve.

## CI

Disabled `.github/workflows` → `.github/workflows.disabled` for local install.

## Remotes

- `origin` — this fork (SSH, push enabled)
- `upstream` — borghei/ink (HTTPS fetch, push DISABLED)
