# Coding Conventions

**Analysis Date:** 2026-01-13

## Naming Patterns

**Files:**
- snake_case for all module files: `manager.rs`, `types.rs`, `renderer.rs`
- `mod.rs` for module exports
- UPPERCASE.md for documentation: `README.md`, `CLAUDE.md`

**Functions:**
- snake_case for all functions: `update_sessions_stable`, `handle_key`, `generate_smart_session_names`
- No special prefix for async functions (Rust convention)
- Descriptive names preferred: `generate_context_aware_name` over `gen_name`

**Variables:**
- snake_case for all variables: `session_name`, `current_length`, `result_segments`
- No underscore prefix convention for private members

**Types:**
- PascalCase for structs: `SessionManager`, `PluginState`, `PluginRenderer`, `ZoxideDirectory`
- PascalCase for enums: `SessionItem`, `SessionAction`, `ActiveScreen`
- No I prefix for traits (Rust convention)

**Constants:**
- UPPER_SNAKE_CASE: `MISSING_THRESHOLD` (`src/session/manager.rs:7`)

## Code Style

**Formatting:**
- rustfmt with default settings (no custom rustfmt.toml)
- Enforced in CI via `cargo fmt --check`
- 4 space indentation (Rust default)

**Linting:**
- Clippy with strict mode: `cargo clippy -- -D warnings`
- Enforced in CI on WASM target (`.github/workflows/ci.yml:53`)
- All warnings treated as errors

## Import Organization

**Order:**
1. Standard library imports (`std::*`)
2. External crate imports (`zellij_tile::*`, `serde::*`)
3. Internal module imports (`crate::*`)

**Grouping:**
- Blank line between groups
- `use` statements at top of file
- Re-exports via `pub use` in `mod.rs` files

**Path Aliases:**
- `crate::` for internal absolute paths
- No custom path aliases

## Error Handling

**Patterns:**
- Use `Option<T>` for nullable values
- No custom error types (domain is small)
- Validation errors shown via UI `.set_error()` method

**Error Types:**
- Throw validation errors to user via `self.set_error()` (`src/state.rs:616, 620, 626`)
- Permission errors handled at plugin level (`src/main.rs:79-86`)
- No panic-inducing `unwrap()` without context

## Logging

**Framework:**
- No external logging framework
- Plugin logs to Zellij's plugin log output

**Patterns:**
- Inline comments for debugging complex logic
- No structured logging

## Comments

**When to Comment:**
- Explain "why" for complex algorithms: `src/main.rs:215-235` (smart naming explanation)
- Document business constraints: `src/main.rs:338-345` (Unix socket path limits)
- Note Zellij-specific behaviors: `CLAUDE.md` (WASM sandbox limitations)

**Doc Comments:**
- `///` doc comments on all public items
- Example: `src/session/manager.rs:24-26`
  ```rust
  /// Update session list with stability tracking
  /// Returns true if the visible session list changed
  ```

**TODO Comments:**
- Format: `// TODO: description`
- Example: `src/new_session_info.rs:271` - `// TODO: merge with similar function`

## Function Design

**Size:**
- Most functions under 50 lines
- Complex algorithms documented inline: `apply_smart_truncation` (62 lines, well-commented)

**Parameters:**
- Use `&self` for methods
- Prefer `&str` over `String` for read-only strings
- Use references to avoid cloning

**Return Values:**
- Explicit return statements
- Use `Option<T>` for nullable returns
- Return `bool` for operations that may or may not change state

## Module Design

**Exports:**
- Named exports via `pub use` in `mod.rs`
- Example from `src/session/mod.rs`:
  ```rust
  pub mod manager;
  pub mod types;

  pub use manager::SessionManager;
  pub use types::{SessionAction, SessionItem};
  ```

**Barrel Files:**
- Each module directory has `mod.rs` for public API
- Internal helpers kept private (not in `pub use`)

## Derive Macros

**Common Patterns:**
- `#[derive(Debug, Clone)]` on data types
- `#[derive(Default)]` when zero-initialization makes sense
- `#[derive(PartialEq)]` for comparison-needed types
- Example: `src/state.rs:40` - `#[derive(Debug, Clone, Copy, Default, PartialEq)]`

## Testing Conventions

**Location:**
- Inline `#[cfg(test)]` modules in source files
- Example: `src/session/manager.rs:201`

**Naming:**
- Test function names describe behavior: `test_new_session_added_immediately`
- Prefix with `test_`

**Structure:**
- Arrange-Act-Assert pattern
- Helper functions for complex setup: `make_session()` in `src/session/manager.rs:205`

---

*Convention analysis: 2026-01-13*
*Update when patterns change*
