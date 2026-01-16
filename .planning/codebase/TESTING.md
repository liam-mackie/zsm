# Testing Patterns

**Analysis Date:** 2026-01-13

## Test Framework

**Runner:**
- Rust built-in test framework
- No external test crate

**Assertion Library:**
- Built-in `assert!`, `assert_eq!`, `assert_ne!`
- Standard Rust matchers

**Run Commands:**
```bash
cargo test                                    # Run all tests (native target)
cargo test --target x86_64-unknown-linux-gnu  # Explicit native target (CI)
cargo test -- --nocapture                     # Show println! output
cargo test test_name                          # Single test
```

**Important Note:**
Tests must run on native target, NOT WASM. The `.cargo/config.toml` sets default target to wasm32-wasip1, so CI explicitly specifies native target.

## Test File Organization

**Location:**
- Inline `#[cfg(test)]` modules in source files
- No separate `tests/` directory

**Naming:**
- Test module: `mod tests { }` within source file
- Test functions: `fn test_<behavior_description>()`

**Structure:**
```
src/
  session/
    manager.rs          # Contains #[cfg(test)] mod tests { 7 tests }
    types.rs            # No tests (simple enum definitions)
  state.rs              # No tests (complex, would benefit from tests)
  main.rs               # No tests (complex smart naming algorithm)
  ui/
    renderer.rs         # No tests (UI rendering)
  zoxide/
    search.rs           # No tests (fuzzy search logic)
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function for test setup
    fn make_session(name: &str, is_current: bool) -> SessionInfo {
        // ... construct complex test object
    }

    #[test]
    fn test_new_session_added_immediately() {
        // arrange
        let mut manager = SessionManager::default();

        // act
        let changed = manager.update_sessions_stable(vec![make_session("test", false)]);

        // assert
        assert!(changed);
        assert_eq!(manager.sessions().len(), 1);
        assert_eq!(manager.sessions()[0].name, "test");
    }
}
```

**Patterns:**
- Arrange-Act-Assert (AAA) pattern
- Helper functions for complex object construction
- Clear test names describing expected behavior
- One assertion focus per test (but multiple `assert!` OK)

## Mocking

**Framework:**
- No mocking framework used
- Test helpers create real objects with test data

**Patterns:**
```rust
// Helper creates SessionInfo with all required fields
fn make_session(name: &str, is_current: bool) -> SessionInfo {
    use std::collections::BTreeMap;
    use zellij_tile::prelude::PaneManifest;
    SessionInfo {
        name: name.to_string(),
        is_current_session: is_current,
        tabs: Vec::new(),
        panes: PaneManifest { panes: BTreeMap::new() },
        connected_clients: 0,
        available_layouts: Vec::new(),
        // ... other required fields
    }
}
```

**What to Mock:**
- SessionInfo objects (via helper functions)
- Not applicable: No external API calls to mock

**What NOT to Mock:**
- Zellij plugin runtime (not available in test context)
- WASM-specific features (tests run on native target)

## Fixtures and Factories

**Test Data:**
- Factory function pattern: `make_session()` in `src/session/manager.rs:205`
- Inline test data for simple cases

**Location:**
- Factory functions in test module alongside tests
- No shared fixtures directory

## Coverage

**Requirements:**
- No enforced coverage target
- Coverage tracked for awareness only

**Configuration:**
- Not explicitly configured
- Could use `cargo tarpaulin` or similar

**Current State:**
- Only `src/session/manager.rs` has tests (7 tests)
- Other modules lack test coverage (state, main, ui, zoxide)

## Test Types

**Unit Tests:**
- Scope: Test single function/method in isolation
- Location: `src/session/manager.rs` (7 tests)
- Focus: `SessionManager::update_sessions_stable()` behavior
- Speed: Fast (<1s total)

**Integration Tests:**
- Not currently implemented
- Would require Zellij plugin test harness

**E2E Tests:**
- Not applicable (plugin runs inside Zellij)
- Manual testing via development layout (`zellij.kdl`)

## Common Patterns

**Testing State Changes:**
```rust
#[test]
fn test_session_removed_after_threshold_missing_updates() {
    let mut manager = SessionManager::default();

    // Add initial session
    manager.update_sessions_stable(vec![make_session("test", false)]);

    // Simulate threshold number of missing updates
    for _ in 0..3 {
        manager.update_sessions_stable(vec![]);
    }

    // Assert session removed
    assert_eq!(manager.sessions().len(), 0);
}
```

**Testing Boolean Returns:**
```rust
#[test]
fn test_is_current_session_update_triggers_change() {
    let mut manager = SessionManager::default();
    manager.update_sessions_stable(vec![make_session("test", false)]);

    // Updating is_current should trigger change
    let changed = manager.update_sessions_stable(vec![make_session("test", true)]);

    assert!(changed);
}
```

**Snapshot Testing:**
- Not used in this codebase
- Would require external crate

## CI Testing

**Pipeline (`.github/workflows/ci.yml`):**
```yaml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test --target x86_64-unknown-linux-gnu
```

**Notes:**
- Tests run on Linux CI (ubuntu-latest)
- Explicit native target required (overrides .cargo/config.toml)
- Separate job for WASM compilation verification

## Test Gaps

**Missing Coverage:**
- `src/state.rs` (709 lines) - Central orchestration logic
- `src/main.rs` (503 lines) - Smart naming algorithm (HIGH PRIORITY)
- `src/ui/renderer.rs` (431 lines) - Rendering logic
- `src/zoxide/search.rs` (204 lines) - Fuzzy search

**Risks:**
- Smart naming algorithm has many edge cases untested
- Session creation flow untested
- UI rendering behavior untested

**Recommendations:**
- Add unit tests for `generate_smart_session_names()` in `src/main.rs`
- Add tests for `SearchEngine` in `src/zoxide/search.rs`
- Consider snapshot tests for complex output

---

*Testing analysis: 2026-01-13*
*Update when test patterns change*
