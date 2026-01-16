# Technology Stack

**Analysis Date:** 2026-01-13

## Languages

**Primary:**
- Rust 2021 edition - All application code (`Cargo.toml`)

**Secondary:**
- None (pure Rust codebase)

## Runtime

**Environment:**
- WASM/WASI target: `wasm32-wasip1` (`.cargo/config.toml`)
- Compiles to WebAssembly for execution within Zellij WASM sandbox
- WASM binary output: `target/wasm32-wasip1/debug/zsm.wasm` or `target/wasm32-wasip1/release/zsm.wasm`

**Package Manager:**
- Cargo - Rust package manager
- Lockfile: `Cargo.lock` present

## Frameworks

**Core:**
- `zellij-tile 0.43.1` - Zellij plugin trait implementation and event handling (`Cargo.toml`)
- `zellij-utils 0.43.1` - Zellij utilities library (`Cargo.toml`)

**Testing:**
- Rust built-in test framework (`#[test]`, `#[cfg(test)]`)
- No external test crate required

**Build/Dev:**
- rustfmt - Code formatting (enforced in CI)
- clippy - Linting with `-D warnings` strict mode
- cargo - Build, test, and package management

## Key Dependencies

**Critical:**
- `fuzzy-matcher 0.3.7` - Fuzzy string matching for directory/session search (`Cargo.toml`)
- `serde 1.0.164` - Serialization framework with derive macros (`Cargo.toml`)

**Infrastructure:**
- `uuid 1.8.0` - UUID generation (v4) for plugin request tracking (`Cargo.toml`)
- `humantime 2.2.0` - Human-readable time formatting (`Cargo.toml`)

## Configuration

**Environment:**
- No environment variables required
- Configuration via Zellij layout options (KDL format)
- Settings passed through plugin configuration map: `default_layout`, `session_separator`, `show_resurrectable_sessions`, `base_paths`, `show_all_sessions`

**Build:**
- `.cargo/config.toml` - Default WASM target configuration
- `Cargo.toml` - Dependency and edition configuration

## Platform Requirements

**Development:**
- macOS/Linux (any platform with Rust toolchain)
- rustup with `wasm32-wasip1` target: `rustup target add wasm32-wasip1`
- zoxide CLI for testing (optional but recommended)

**Production:**
- Runs inside Zellij terminal multiplexer
- Distributed as `.wasm` binary via GitHub Releases
- SHA256 checksum provided for release verification

---

*Stack analysis: 2026-01-13*
*Update after major dependency changes*
